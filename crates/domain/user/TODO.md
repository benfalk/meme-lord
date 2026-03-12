# `TODO`

- [X] Setup `fake` for all data types and entities in this crate.
  - [X] Update tests to use `fake` data instead of hard-coded values.
- [ ] Ensure errors from commands and queries can be converted to `crate::Error`
- [ ] Create a `ChangePassword` command that requires the user's current
      password for verification before allowing the password change.
- [ ] Create an `EventPublisher` port trait to allow for publishing events
      to an event bus or message queue.
  - [ ] Create events for all of the commands in this crate
