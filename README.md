# git commit engine (git-ce)

A simple CLI for working with conventional commits.

To get started, download the latest release and install it in your PATH. All releases >= v0.3.6 will be signed with the following minisign public key: `RWQDeHJmW6FpoibSv2+6BuKqQ+n8bvOePFMSXv2s0dvw5k/5wuOffbhT`.


Once installed, all you have to do is run in a repo with added changes:
```sh
git ce 
```

## Adding custom scopes
We use the Git config to set scopes. To do this you would just run:
```sh
git config ce.flake <scope> --add
```
