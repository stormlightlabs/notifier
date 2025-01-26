# Stormlight Notifier

Custom discord bot to post updates to the Stormlight labs discord server

## TODO

- Implement Discord connection
- Create axum server
  - Root endpoint for status
  - Webhook endpoint
- Add structured logging
- Implement GitHub event parsing
  - Push
    - author, branch, commits
  - PR
    - title, author, description
  - Issues
    - title, author, labels
- Design Discord embed messages
- Add channel ID mapping
- Implement rate limiting
