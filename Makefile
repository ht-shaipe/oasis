# Makefile for Oasis (Tauri + Bun)

.PHONY: dev build bundle install git help

# Default target
help:
	@echo "Available commands:"
	@echo "  make dev      - Start Tauri development mode (bun tauri dev)"
	@echo "  make build    - Build frontend code (bun run build)"
	@echo "  make bundle   - Execute Tauri build packaging (bun tauri build)"
	@echo "  make install  - Install all dependencies (bun install)"
	@echo "  make git      - Add, commit and push changes"

# Start Tauri development mode
dev:
	bun run tauri

# Build frontend code
build:
	bun run build

# Execute Tauri build packaging
bundle:
	bun run tauri:build

# Install all dependencies
install:
	bun install

# git commit and push logic
ifneq ($(filter git,$(MAKECMDGOALS)),)
  GIT_MSG_ARGS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
  $(foreach _g,$(GIT_MSG_ARGS),$(eval $(_g):;@:))
endif

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
	echo "git commit and push success"
