# git commit and push

ifneq ($(filter git,$(MAKECMDGOALS)),)
  GIT_MSG_ARGS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
  $(foreach _g,$(GIT_MSG_ARGS),$(eval $(_g):;@:))
endif

.PHONY: git install-web build-wasm build-wasm-release dev dev-web build-web preview-web clean-web

# git commit and push
git:
	@set -e; \
	msg=''; \
	if [ -n "$(strip $(MSG))" ]; then \
		msg='$(subst ','\'',$(MSG))'; \
	elif [ -n "$(strip $(GIT_MSG_ARGS))" ]; then \
		msg='$(subst ','\'',$(GIT_MSG_ARGS))'; \
	else \
		printf 'input commit message: '; read -r msg; \
	fi; \
	git add . && \
	git commit -a -m "$$msg" && \
	git pull && \
	git push && \
	echo git commit and push success

## Web (WASM + Vite): requires Rust nightly + wasm32 + wasm-bindgen-cli; frontend uses bun.

install-web:
	@rustup toolchain install nightly --component rustfmt 2>/dev/null || true
	@rustup target add wasm32-unknown-unknown --toolchain nightly 2>/dev/null || true
	@command -v wasm-bindgen >/dev/null 2>&1 || cargo install wasm-bindgen-cli --version 0.2.121 -f
	@cd www && bun install

build-wasm:
	@./scripts/build-wasm.sh

build-wasm-release:
	@./scripts/build-wasm.sh --release

dev:
	@set -e; \
	cargo build --workspace --lib --exclude wasm-plugin; \
	./crates/widgets/wasm-widget/build.sh; \
	copy_plugin() { \
		src="$$1"; \
		dst_dir="$$2"; \
		dst_name="$$3"; \
		if [ -f "$$src" ]; then \
			mkdir -p "$$dst_dir"; \
			cp "$$src" "$$dst_dir/$$dst_name"; \
			echo "copied $$src -> $$dst_dir/$$dst_name"; \
		fi; \
	}; \
	ensure_assets() { \
		dst_dir="$$1"; \
		crate_dir="$$2"; \
		plugin_id="$$3"; \
		display_name="$$4"; \
		desc="$$5"; \
		if [ ! -f "$$dst_dir/icon.svg" ] && [ -f "$$crate_dir/icon.svg" ]; then \
			mkdir -p "$$dst_dir"; \
			cp "$$crate_dir/icon.svg" "$$dst_dir/icon.svg"; \
			echo "copied $$crate_dir/icon.svg -> $$dst_dir/icon.svg"; \
		fi; \
		if [ ! -f "$$dst_dir/manifest.toml" ]; then \
			if [ -f "$$crate_dir/manifest.toml" ]; then \
				cp "$$crate_dir/manifest.toml" "$$dst_dir/manifest.toml"; \
				echo "copied $$crate_dir/manifest.toml -> $$dst_dir/manifest.toml"; \
			else \
				mkdir -p "$$dst_dir"; \
				printf '[plugin]\nid = "%s"\ndisplay_name = "%s"\ndescription = "%s"\nicon = "icon.svg"\nwindow_width = 800.0\nwindow_height = 600.0\n' "$$plugin_id" "$$display_name" "$$desc" > "$$dst_dir/manifest.toml"; \
				echo "generated $$dst_dir/manifest.toml"; \
			fi; \
		fi; \
	}; \
	rm -f plugins/md-editor-plugin/md-editor-plugin.dylib; \
	copy_plugin target/debug/libmd-editor-plugin.dylib plugins/md-editor-plugin md-editor-plugin.dylib; \
	rm -f plugins/notepad/notepad.dylib; \
	copy_plugin target/debug/libnotepad_plugin.dylib plugins/notepad notepad.dylib; \
	rm -f plugins/toolbox/toolbox.dylib; \
	copy_plugin target/debug/libtoolbox_plugin.dylib plugins/toolbox toolbox.dylib; \
	rm -f plugins/credential/credential.dylib; \
	copy_plugin target/debug/libcredential_plugin.dylib plugins/credential credential.dylib; \
	ensure_assets plugins/md-editor-plugin crates/plugins/md-editor-plugin md-editor-plugin "Markdown编辑器" "Markdown文档编辑与预览"; \
	ensure_assets plugins/notepad crates/plugins/notepad-plugin notepad "记事本" "轻量级文本编辑器"; \
	ensure_assets plugins/toolbox crates/plugins/toolbox-plugin toolbox "工具箱" "实用工具集：CSV统计/分割、批量重命名、网络扫描等"; \
	ensure_assets plugins/credential crates/plugins/credential-plugin credential "凭证管理" "密码与密钥管理工具"; \
	cargo r

dev-web: build-wasm
	@cd www && bun install && bun run dev

build-web: build-wasm-release
	@cd www && bun install && bun run build

preview-web:
	@cd www && bun run preview

clean-web:
	@rm -rf www/dist www/src/wasm/*.js www/src/wasm/*.wasm
