# `TODO`

- [X] Setup `fake` for all data types and entities in this crate.
  - [X] Update tests to use `fake` data instead of hard-coded values.
- [X] Ensure errors from commands and queries can be converted to `crate::Error`
- [X] Create a `ChangePassword` command that requires the user's current
      password for verification before allowing the password change.
- [X] Create an `EventPublisher` port trait to allow for publishing events
      to an event bus or message queue.
  - [X] Create events for all of the commands in this crate
- [ ] Create a `Null` event publisher implementation that does nothing, in case
      we don't want to publish events in certain environments (e.g. testing).
