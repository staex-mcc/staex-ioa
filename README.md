# Staex IoA

Staex Internet of Applications.


subxt metadata --url https://rpcpc1-qa.agung.peaq.network -f json -o ./provisioner/asd.json

subxt codegen --file ../../provisioner/assets/$(name).metadata.scale \
		--no-docs | \
		rustfmt --edition=2021 --emit=stdout > ../../provisioner/src/$(name).rs
