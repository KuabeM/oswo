# OSWO - organize sway outputs

Organize your monitors from the cli.

## Get started

Check `oswo display` to get an overview of available montiors. Copy the [`cfgs.toml`] file to
`$XDG_CONFIG_DIR/oswo.toml` and replace the top level names with the desired setups you want to
have, e.g. `alone` for just the laptop monitor or `office` for your office setup. The names are
the model string as reported by `oswo display`.

To use one of the configured setups, use `oswo use <name>`, e.g. `oswo use alone`.

If you just want to configure monitors by their enumerated output identifier use `oswo set eDP-1`
with the respective identifier as reported by `oswo display`.
