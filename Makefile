.PHONY: watch
watch:
	watchexec -r --stop-signal SIGINT cargo run -- op-fuse.toml
