# This is a generically useful Makefile for F* that is self-contained
#
# We expect:
#  1. `fstar.exe` to be in PATH (alternatively, you can also set
#     $FSTAR_HOME to be set to your F* repo/install directory)
#
#  2. `cargo`, `rustup`, `hax` and `jq` to be installed and in PATH.
#
#  3. the extracted Cargo crate to have "hax-lib" as a dependency:
#     `hax-lib = { version = "0.1.0-pre.1", git = "https://github.com/hacspec/hax"}`
#
# Optionally, you can set `HACL_HOME`.
#
# ROOTS contains all the top-level F* files you wish to verify
# The default target `verify` verified ROOTS and its dependencies
# To lax-check instead, set `OTHERFLAGS="--lax"` on the command-line
#
# To make F* emacs mode use the settings in this file, you need to
# add the following lines to your .emacs
#
# (setq-default fstar-executable "<YOUR_FSTAR_HOME>/bin/fstar.exe")
# (setq-default fstar-smt-executable "<YOUR_Z3_HOME>/bin/z3")
#
# (defun my-fstar-compute-prover-args-using-make ()
#   "Construct arguments to pass to F* by calling make."
#   (with-demoted-errors "Error when constructing arg string: %S"
#     (let* ((fname (file-name-nondirectory buffer-file-name))
# 	   (target (concat fname "-in"))
# 	   (argstr (car (process-lines "make" "--quiet" target))))
#       (split-string argstr))))
# (setq fstar-subp-prover-args #'my-fstar-compute-prover-args-using-make)
#

PATH_TO_CHILD_MAKEFILE := "$(abspath $(firstword $(MAKEFILE_LIST)))"
PATH_TO_TEMPLATE_MAKEFILE := "$(abspath $(lastword $(MAKEFILE_LIST)))"

HACL_HOME      ?= $(HOME)/.hax/hacl_home
# Expand variable FSTAR_BIN_DETECT now, so that we don't run this over and over

FSTAR_BIN_DETECT := $(if $(shell command -v fstar.exe), fstar.exe, $(FSTAR_HOME)/bin/fstar.exe)
FSTAR_BIN      ?= $(FSTAR_BIN_DETECT)

GIT_ROOT_DIR   := $(shell git rev-parse --show-toplevel)/
CACHE_DIR      ?= ${GIT_ROOT_DIR}.fstar-cache/checked
HINT_DIR       ?= ${GIT_ROOT_DIR}.fstar-cache/hints

# Makes command quiet by default
Q ?= @

# Verify the required executable are in PATH
EXECUTABLES = cargo cargo-hax jq
K := $(foreach exec,$(EXECUTABLES),\
        $(if $(shell which $(exec)),some string,$(error "No $(exec) in PATH")))

export ANSI_COLOR_BLUE=\033[34m
export ANSI_COLOR_RED=\033[31m
export ANSI_COLOR_BBLUE=\033[1;34m
export ANSI_COLOR_GRAY=\033[90m
export ANSI_COLOR_TONE=\033[35m
export ANSI_COLOR_RESET=\033[0m

ifdef NO_COLOR
export ANSI_COLOR_BLUE=
export ANSI_COLOR_RED=
export ANSI_COLOR_BBLUE=
export ANSI_COLOR_GRAY=
export ANSI_COLOR_TONE=
export ANSI_COLOR_RESET=
endif

# The following is a bash script that discovers F* libraries.
# Due to incompatibilities with make 4.3, I had to make a "oneliner" bash script...
define FINDLIBS
    : "Prints a path if and only if it exists. Takes one argument: the path."; \
    function print_if_exists() { \
        if [ -d "$$1" ]; then \
            echo "$$1"; \
        fi; \
    } ; \
    : "Asks Cargo all the dependencies for the current crate or workspace,"; \
    : "and extract all "root" directories for each. Takes zero argument."; \
    function dependencies() { \
        cargo metadata --format-version 1 | \
            jq -r ".packages | .[] | .manifest_path | split(\"/\") | .[:-1] | join(\"/\")"; \
    } ; \
    : "Find hax libraries *around* a given path. Takes one argument: the"; \
    : "path."; \
    function find_hax_libraries_at_path() { \
        path="$$1" ; \
        : "if there is a [proofs/fstar/extraction] subfolder, then that s a F* library" ; \
        print_if_exists "$$path/proofs/fstar/extraction" ; \
        : "Maybe the [proof-libs] folder of hax is around?" ; \
        MAYBE_PROOF_LIBS=$$(realpath -q "$$path/../proof-libs/fstar") ; \
        if [ $$? -eq 0 ]; then \
            print_if_exists "$$MAYBE_PROOF_LIBS/core" ; \
            print_if_exists "$$MAYBE_PROOF_LIBS/rust_primitives" ; \
        fi ; \
    } ; \
    { while IFS= read path; do \
          find_hax_libraries_at_path "$$path"; \
      done < <(dependencies) ; } | sort -u
endef
export FINDLIBS

FSTAR_INCLUDE_DIRS_EXTRA ?=
FINDLIBS_OUTPUT := $(shell bash -c '${FINDLIBS}')
FSTAR_INCLUDE_DIRS = $(HACL_HOME)/lib $(FSTAR_INCLUDE_DIRS_EXTRA) $(FINDLIBS_OUTPUT)

# Make sure FSTAR_INCLUDE_DIRS has the `proof-libs`, print hints and
# an error message otherwise
ifneq (,$(findstring proof-libs/fstar,$(FSTAR_INCLUDE_DIRS)))
else
	K += $(info )
	ERROR := $(shell printf '${ANSI_COLOR_RED}Error: could not detect `proof-libs`!${ANSI_COLOR_RESET}')
	K += $(info ${ERROR})
	ERROR := $(shell printf '  > Do you have `${ANSI_COLOR_BLUE}hax-lib${ANSI_COLOR_RESET}` in your `${ANSI_COLOR_BLUE}Cargo.toml${ANSI_COLOR_RESET}` as a ${ANSI_COLOR_BLUE}git${ANSI_COLOR_RESET} or ${ANSI_COLOR_BLUE}path${ANSI_COLOR_RESET} dependency?')
	K += $(info ${ERROR})
	ERROR := $(shell printf '  ${ANSI_COLOR_BLUE}> Tip: you may want to run `cargo add --git https://github.com/hacspec/hax hax-lib`${ANSI_COLOR_RESET}')
	K += $(info ${ERROR})
	K += $(info )
	K += $(error Fatal error: `proof-libs` is required.)
endif

.PHONY: all verify clean

all:
	$(Q)rm -f .depend
	$(Q)$(MAKE) .depend hax.fst.config.json verify

all-keep-going:
	$(Q)rm -f .depend
	$(Q)$(MAKE) --keep-going .depend hax.fst.config.json verify

# If $HACL_HOME doesn't exist, clone it
${HACL_HOME}:
	$(Q)mkdir -p "${HACL_HOME}"
	$(info Clonning Hacl* in ${HACL_HOME}...)
	git clone --depth 1 https://github.com/hacl-star/hacl-star.git "${HACL_HOME}"
	$(info Clonning Hacl* in ${HACL_HOME}... done!)

# If no any F* file is detected, we run hax
ifeq "$(wildcard *.fst *fsti)" ""
$(shell cargo hax into fstar)
endif

# By default, we process all the files in the current directory
ROOTS ?= $(wildcard *.fst *fsti)
ADMIT_MODULES ?=

ADMIT_MODULE_FLAGS ?= --admit_smt_queries true

# Can be useful for debugging purposes
FINDLIBS.sh:
	$(Q)echo '${FINDLIBS}' > FINDLIBS.sh
include-dirs:
	$(Q)bash -c '${FINDLIBS}'

FSTAR_FLAGS = \
  --warn_error -321-331-241-274-239-271 \
  --cache_checked_modules --cache_dir $(CACHE_DIR) \
  --already_cached "+Prims+FStar+LowStar+C+Spec.Loops+TestLib" \
  $(addprefix --include ,$(FSTAR_INCLUDE_DIRS))

FSTAR := $(FSTAR_BIN) $(FSTAR_FLAGS)

.depend: $(HINT_DIR) $(CACHE_DIR) $(ROOTS)
	@$(FSTAR) --dep full $(ROOTS) --extract '* -Prims -LowStar -FStar' > $@

include .depend

$(HINT_DIR) $(CACHE_DIR):
	$(Q)mkdir -p $@

define HELPMESSAGE
echo "hax' default Makefile for F*"
echo ""
echo "The available targets are:"
echo ""
function target() {
  printf '  ${ANSI_COLOR_BLUE}%-20b${ANSI_COLOR_RESET} %s\n' "$$1" "$$2"
}
target "all" "Verify every F* files (stops whenever an F* fails first)"
target "all-keep-going" "Verify every F* files (tries as many F* module as possible)"
target "" ""
target "run/${ANSI_COLOR_TONE}<MyModule.fst>  " 'Runs F* on `MyModule.fst` only'
target "" ""
target "vscode" 'Generates a `hax.fst.config.json` file'
target "${ANSI_COLOR_TONE}<MyModule.fst>${ANSI_COLOR_BLUE}-in   " 'Useful for Emacs, outputs the F* prefix command to be used'
target "" ""
target "clean" 'Cleanup the target'
target "include-dirs" 'List the F* include directories'
target "" ""
target "describe" 'List the F* root modules, and describe the environment.'
echo ""
echo "Variables:"
target "NO_COLOR" "Set to anything to disable colors"
target "ADMIT_MODULES" "List of modules where F* will assume every SMT query"
target "FSTAR_INCLUDE_DIRS_EXTRA" "List of extra include F* dirs"
endef
export HELPMESSAGE

describe:
	@printf '${ANSI_COLOR_BBLUE}F* roots:${ANSI_COLOR_RESET}\n'
	@for root in ${ROOTS}; do \
	  filename=$$(basename -- "$$root") ;\
	  ext="$${filename##*.}" ;\
	  noext="$${filename%.*}" ;\
	  printf "${ANSI_COLOR_GRAY}$$(dirname -- "$$root")/${ANSI_COLOR_RESET}%s${ANSI_COLOR_GRAY}.${ANSI_COLOR_TONE}%s${ANSI_COLOR_RESET}%b\n" "$$noext" "$$ext" $$([[ "${ADMIT_MODULES}" =~ (^| )$$root($$| ) ]] && echo '${ANSI_COLOR_RED}\t[ADMITTED]${ANSI_COLOR_RESET}'); \
	done
	@printf '\n${ANSI_COLOR_BBLUE}Environment:${ANSI_COLOR_RESET}\n'
	@printf ' - ${ANSI_COLOR_BLUE}HACL_HOME${ANSI_COLOR_RESET} = %s\n' '${HACL_HOME}'
	@printf ' - ${ANSI_COLOR_BLUE}FSTAR_BIN${ANSI_COLOR_RESET} = %s\n' '${FSTAR_BIN}'
	@printf ' - ${ANSI_COLOR_BLUE}GIT_ROOT_DIR${ANSI_COLOR_RESET} = %s\n' '${GIT_ROOT_DIR}'
	@printf ' - ${ANSI_COLOR_BLUE}CACHE_DIR${ANSI_COLOR_RESET} = %s\n' '${CACHE_DIR}'
	@printf ' - ${ANSI_COLOR_BLUE}HINT_DIR${ANSI_COLOR_RESET} = %s\n' '${HINT_DIR}'
	@printf ' - ${ANSI_COLOR_BLUE}ADMIT_MODULE_FLAGS${ANSI_COLOR_RESET} = %s\n' '${ADMIT_MODULE_FLAGS}'
	@printf ' - ${ANSI_COLOR_BLUE}FSTAR_INCLUDE_DIRS_EXTRA${ANSI_COLOR_RESET} = %s\n' '${FSTAR_INCLUDE_DIRS_EXTRA}'

help: ;@bash -c "$$HELPMESSAGE"
h: ;@bash -c "$$HELPMESSAGE"

HEADER = $(Q)printf '${ANSI_COLOR_BBLUE}[CHECK] %s ${ANSI_COLOR_RESET}\n' "$(basename $(notdir $@))"

run/%: | .depend $(HINT_DIR) $(CACHE_DIR)
	${HEADER}
	$(Q)$(FSTAR) $(OTHERFLAGS) $(@:run/%=%)

VERIFIED_CHECKED = $(addsuffix .checked, $(addprefix $(CACHE_DIR)/,$(ROOTS)))
ADMIT_CHECKED = $(addsuffix .checked, $(addprefix $(CACHE_DIR)/,$(ADMIT_MODULES)))

$(ADMIT_CHECKED):
	$(Q)printf '${ANSI_COLOR_BBLUE}[${ANSI_COLOR_TONE}ADMIT${ANSI_COLOR_BBLUE}] %s ${ANSI_COLOR_RESET}\n' "$(basename $(notdir $@))"
	$(Q)$(FSTAR) $(OTHERFLAGS) $(ADMIT_MODULE_FLAGS) $< $(ENABLE_HINTS) --hint_file $(HINT_DIR)/$(notdir $*).hints || { \
	  echo "" ; \
	  exit 1 ; \
	}
	$(Q)printf "\n\n"

$(CACHE_DIR)/%.checked: | .depend $(HINT_DIR) $(CACHE_DIR)
	${HEADER}
	$(Q)$(FSTAR) $(OTHERFLAGS) $< $(ENABLE_HINTS) --hint_file $(HINT_DIR)/$(notdir $*).hints || { \
	  echo "" ; \
	  exit 1 ; \
	}
	touch $@
	$(Q)printf "\n\n"

verify: $(VERIFIED_CHECKED) $(ADMIT_CHECKED)

# Targets for Emacs
%.fst-in:
	$(info $(FSTAR_FLAGS) \
	  $(ENABLE_HINTS) --hint_file $(HINT_DIR)/$(basename $@).fst.hints)
%.fsti-in:
	$(info $(FSTAR_FLAGS) \
	  $(ENABLE_HINTS) --hint_file $(HINT_DIR)/$(basename $@).fsti.hints)

# Targets for VSCode
hax.fst.config.json: .depend
	$(Q)echo "$(FSTAR_INCLUDE_DIRS)" | jq --arg fstar "$(FSTAR_BIN)" -R 'split(" ") | {fstar_exe: $$fstar | gsub("^\\s+|\\s+$$";""), include_dirs: .}' > $@
vscode:
	$(Q)rm -f .depend
	$(Q)$(MAKE) hax.fst.config.json

SHELL=bash

# Clean target
clean:
	rm -rf $(CACHE_DIR)/*
	rm *.fst
