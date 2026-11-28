// The notices are written by scripts/third-party.mjs in a small part of Markdown: headings,
// paragraphs, pipe tables, fenced text, inline code and links. It is drawn as elements and text,
// never as HTML, so nothing a licence says can turn into markup.

const LEADS_OUT = /^(https?:\/\/|mailto:)/i;
// Code, a link whose address may hold one level of parentheses, and a bare web address, which
// gives up the punctuation that ends a sentence.
const INLINE =
  /`([^`]+)`|\[([^\]]+)\]\(((?:[^()\s]|\([^()\s]*\))+)\)|(https?:\/\/[^\s<>()`]*[^\s<>()`.,;:!?'"])/g;

function made<K extends keyof HTMLElementTagNameMap>(tag: K, children: (Node | string)[] = []) {
  const one = document.createElement(tag);
  one.append(...children);
  return one;
}

function leading(href: string, words: (Node | string)[]) {
  const link = made("a", words);
  link.href = href;
  return link;
}

function inline(said: string, bare = true): (Node | string)[] {
  const out: (Node | string)[] = [];
  let at = 0;
  for (const found of said.matchAll(INLINE)) {
    out.push(said.slice(at, found.index));
    const [whole, code, words, href, address] = found;
    if (code !== undefined) {
      out.push(made("code", [code]));
    } else if (address !== undefined) {
      out.push(bare ? leading(address, [address]) : address);
    } else if (LEADS_OUT.test(href)) {
      out.push(leading(href, inline(words, false)));
    } else {
      // A link to anywhere else keeps its words and loses the address.
      out.push(...inline(words, false));
    }
    at = found.index + whole.length;
  }
  out.push(said.slice(at));
  return out.filter((part) => part !== "");
}

const cells = (row: string) =>
  row
    .trim()
    .replace(/^\|/, "")
    .replace(/\|$/, "")
    .split("|")
    .map((cell) => cell.trim());

function table(rows: string[]) {
  const [head, , ...body] = rows;
  const row = (said: string, tag: "th" | "td") =>
    made(
      "tr",
      cells(said).map((cell) => made(tag, inline(cell))),
    );
  return made("table", [
    made("thead", [row(head, "th")]),
    made(
      "tbody",
      body.map((one) => row(one, "td")),
    ),
  ]);
}

export function composed(text: string): DocumentFragment {
  const page = document.createDocumentFragment();
  const lines = text.replace(/<!--[\s\S]*?-->\s*/g, "").split(/\r?\n/);
  let paragraph: string[] = [];
  const close = () => {
    if (paragraph.length > 0) page.append(made("p", inline(paragraph.join("\n"))));
    paragraph = [];
  };
  for (let at = 0; at < lines.length; at++) {
    const line = lines[at];
    const fence = /^(`{3,})/.exec(line)?.[1];
    const heading = /^(#{1,6})\s+(.*)$/.exec(line);
    if (fence) {
      close();
      // Only a run of backticks as long as the opening one closes it, as in CommonMark.
      const closes = (one: string) => /^`+\s*$/.test(one) && one.trim().length >= fence.length;
      const said: string[] = [];
      for (at++; at < lines.length && !closes(lines[at]); at++) said.push(lines[at]);
      page.append(made("pre", [made("code", [`${said.join("\n")}\n`])]));
    } else if (heading) {
      close();
      const level = heading[1].length as 1 | 2 | 3 | 4 | 5 | 6;
      page.append(made(`h${level}`, inline(heading[2].trim())));
    } else if (line.trimStart().startsWith("|") && lines[at + 1]?.trimStart().startsWith("|")) {
      close();
      const rows: string[] = [];
      for (; at < lines.length && lines[at].trimStart().startsWith("|"); at++) rows.push(lines[at]);
      at--;
      page.append(table(rows));
    } else if (line.trim() === "") {
      close();
    } else {
      paragraph.push(line.trim());
    }
  }
  close();
  return page;
}
