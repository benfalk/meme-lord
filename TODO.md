# `TODO`

- [X] Setup `fake` for all data types and entities in this crate.
  - [X] Update tests to use `fake` data instead of hard-coded values.
- [X] Ensure errors from commands and queries can be converted to `crate::Error`
- [X] Create a `ChangePassword` command that requires the user's current
      password for verification before allowing the password change.
- [X] Create an `EventPublisher` port trait to allow for publishing events
      to an event bus or message queue.
  - [X] Create events for all of the commands in this crate
- [X] Create a `Null` event publisher implementation that does nothing, in case
      we don't want to publish events in certain environments (e.g. testing).
- [ ] Create a SQLX multi-crate repository implementation for the `UserRepository`
      port trait.  This should accept a `sqlx::PgPool` **OR** a database URL as
      a constructor argument.
  - [ ] Should expose some kind of migration functionality to create the
        necessary tables and indexes in the database.
- [X] Create an `identity` domain crate which can store ids for various
      entities from different domains. This will allow us to have a single
      source of truth for these ids and prevent crates from needing to depend
      on each other just to access these ids.
- [X] Create a `meme` domain that allows users to create memes with captions
      and images. This should include commands for creating, updating, and
      deleting memes. It also needs queries by various criteria (e.g. by user,
      by caption, etc.).
  - [X] Scaffold out a new domain with the same basic starting structure as
      the `user` domain.
    - [X] Upload port to store memes in some kind of storage
    - [X] Repository port for memes
    - [X] Events for memes
  - [ ] What crates are best to work with images in Rust?  We need to be able
        to pull data from these images to extract metadata as well as provide
        some editing capabilities such as resizing, cropping, and adding text
        captions.
- [ ] Clean up unwraps in sqlite adapter and replace them with proper error
      handling that converts to errors to avoid panics.
