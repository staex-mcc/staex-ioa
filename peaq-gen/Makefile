update:
	subxt metadata -f bytes --url https://rpcpc1-qa.agung.peaq.network \
		> assets/peaq.metadata.scale
	subxt metadata -f json --url https://rpcpc1-qa.agung.peaq.network \
		> assets/peaq.metadata.json
	subxt codegen --file assets/peaq.metadata.scale \
		--no-docs | \
		rustfmt +nightly --edition=2021 --emit=stdout > src/metadata.rs
