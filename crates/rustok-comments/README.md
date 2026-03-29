# rustok-comments

## Purpose

`rustok-comments` owns the generic comments domain for RusToK.

## Responsibilities

- Provide a dedicated storage boundary for classic comments outside the forum domain.
- Serve as the canonical storage owner for blog comments and other classic non-forum comments.
- Keep `comments` separate from forum topics and forum replies.
- Expose module metadata, permissions, and future migrations for the comments domain.
- Align comment-body contracts with shared rich-text rules from `rustok-content`.
- Reuse shared locale fallback semantics from `rustok-content` so comment reads match blog/pages behavior.

## Interactions

- Depends on `rustok-core` for module contracts and permission vocabulary.
- Depends on `rustok-content` for shared rich-text and locale-resolution helpers.
- Is intended to integrate with `rustok-blog` and `rustok-pages`.
- Must not become the storage backend for `rustok-forum`.

## Entry points

- `CommentsModule`

See also `docs/README.md`.
