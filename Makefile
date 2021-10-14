.PHONY: clean aws_parse upload_parse invoke_parse log_parse

.SECONDARY:

dests = aws_parse

aws_parse_fn := $(AWS_PARSE)

tg_musl = x86_64-unknown-linux-musl
rl_dir = target/$(tg_musl)/release

dist = dist
aws_out := $(dist)/aws_out
aws_log := $(dist)/aws_log
aws_parse_event := $(dist)/aws_parse_event.json

$(rl_dir)/%: src/*.rs src/bin/*.rs
	cargo build --release --bin $(@F) --target $(tg_musl)

$(dist)/%/bootstrap: $(rl_dir)/%_fn
	mkdir -p $(@D)
	cp $< $@

$(dist)/%/app.zip: $(dist)/%/bootstrap
	zip -j $@ $<

$(dests): %: $(dist)/%/app.zip

upload_parse: $(dist)/aws_parse/app.zip
	aws lambda update-function-code --function-name $(AWS_PARSE) --zip-file fileb://$<

invoke_parse:
	aws lambda invoke --function-name $(AWS_PARSE) $(aws_out) \
	--output text --payload fileb://$(aws_parse_event) \
	--log-type Tail > $(aws_log)

log_parse:
	grep -oE '\S{20,}' $(aws_log)| base64 -d
	cat $(aws_out)

clean:
	cargo clean
	rm -rf dist/*
