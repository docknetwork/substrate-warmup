# Substrate proxy

re-exports workspace members from substrate. Workspace members in this repository should refer to
substrate by path to this directory.

```toml
[dependencies]
substrate = { path = "../../substrate-proxy" }
```

This `substrate-proxy` module obviates the old de-facto method of specifying substrate dependencies.

```
# No longer needed:
[dependencies.offchain-primitives]
git = "https://github.com/paritytech/substrate.git"
rev = "870b976bec729aaf26cc237df9fd764b8f7b9d7e"
package = "substrate-offchain-primitives"
default-features = false

# ...
```

This crate/library/workspace member has a feature "std", enabled byt default. For no_std, you'll need to disable it
in the de-facto standard way.

```toml
[dependencies]
substrate = { path = "../../substrate-proxy", default-features = false }

[features]
std = [
	"substrate/std",
]
```
