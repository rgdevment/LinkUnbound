# Commercial licensing

LinkUnbound is dual licensed.

- **GPL-3.0** — free for everyone, forever. See [LICENSE](LICENSE).
- **Commercial licence** — for organisations that cannot accept the GPL's
  terms. Contact <github@apirest.cl>.

Same software, same features. The only thing you buy is a different set of
obligations.

## Do you actually need one?

Almost certainly not. The GPL asks something of you when you **distribute**
the software or a work derived from it. Using it does not trigger anything, no
matter how many people use it or for how long.

**You do not need a commercial licence to:**

- Install and use LinkUnbound at home or at work, on any number of machines.
- Deploy it across an entire organisation, including internal IT rollouts.
- Read, audit, or fork the source.
- Modify it for your own internal use, without publishing those changes.
- Run modified code as an internal service — the GPL has no network clause.
- Contribute changes back.

If you are a company wondering whether rolling this out to your staff needs a
licence: it does not. Internal use is not distribution.

**You likely do need one to:**

- Ship LinkUnbound, or code derived from it, inside a **product you distribute
  to others**, without licensing that product under the GPL.
- Redistribute it under **your own brand** without the GPL's source disclosure
  requirements.
- Bundle it with hardware or preinstall it on machines you sell.
- Satisfy a policy or contract that **forbids copyleft** dependencies in what
  you ship.

If you are unsure which side you fall on, ask. A short description of what you
intend to do is usually enough to answer it, and the answer is often "you are
fine, carry on".

## What a commercial licence gives you

- The right to use, modify, and redistribute LinkUnbound **without the GPL's
  copyleft obligations** — no requirement to publish your modifications or to
  license your own product under the GPL.
- Written permission you can hand to your legal or procurement team.

It covers LinkUnbound's own code and nothing else. The components it is built
on keep their own licences, and the notices those licences ask for still go
with every copy: see [Third-party components](#third-party-components).

Terms, scope, and price are agreed per case rather than published, because a
single-seat integration and an OEM redistribution are not the same deal.
Contact <github@apirest.cl> with what you want to do and the scale of it.

## Third-party components

LinkUnbound is built on work it does not own, and no licence from this project
can relicense it. Everything that ships inside the binaries is listed, with its
version and licence, in [THIRD-PARTY-BUNDLED.md](THIRD-PARTY-BUNDLED.md). Most
of it is MIT, Apache-2.0, BSD or ISC: free to redistribute inside a closed
product, as long as its notices travel with it. Two need a closer look.

**Slint** draws the picker. SixtyFPS GmbH offers it under the GPL-3.0, under
the Slint Royalty-free Desktop, Mobile, and Web Applications License 2.0, or
under a paid commercial licence. A product that takes LinkUnbound under a
commercial licence uses Slint under the Royalty-free licence: no charge, on
condition that the product shows Slint's `AboutSlint` widget in its About
screen, or the
[#MadeWithSlint badge](https://github.com/slint-ui/slint/tree/master/logo) on
the public page its binaries are downloaded from. That licence does not cover
embedded systems, nor a product that exposes Slint's API to its own users;
those need Slint's commercial licence, bought from SixtyFPS, not from this
project.

**MPL-2.0** covers a few crates in the build (`cssparser`, `cssparser-macros`, `selectors`,
`dtoa-short`, `option-ext`). The MPL is copyleft per file: the source of those
files, and of any change made to them, stays available under the MPL, and the
rest of the product is untouched. LinkUnbound ships them unmodified.

## Store distribution

The GPL's section 10 forbids imposing further restrictions on recipients,
which conflicts with the terms of some application stores — Apple's App Store
being the well-known case. That conflict binds **licensees**, not the
copyright holder: a third party cannot publish LinkUnbound there, and the
project itself can, under separate terms it grants to itself. It holds that
right over its own code only. For Slint the project is a licensee like anyone
else, so a build for such a store has to take Slint under its Royalty-free
licence instead of the GPL, with the attribution that licence asks for.

In practice today: macOS builds are distributed as a signed and notarised DMG
under the GPL, and the Microsoft Store build is unaffected, since Microsoft's
terms defer to the software's own licence.

## Why the project is set up this way

The dual model exists so LinkUnbound can stay genuinely free for the people
who use it, while commercial redistribution that would otherwise contribute
nothing back has a way to support it. Whichever side you are on, the GPL
edition is not a crippled version — it is the same application, and it stays
that way.

## For contributors

Dual licensing only works if the project can license every merged line under
both sets of terms, which is why contributions require a signed
[CLA](CLA.md). You keep the copyright on your work, and the commitments in
section 6 of that document bind the project in return. See
[CONTRIBUTING.md](CONTRIBUTING.md).
