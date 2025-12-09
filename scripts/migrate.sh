#!/usr/bin/env bash

# ******************************************************************
# * WARNING: most of this file was written with the help of an LLM *
# ******************************************************************

if [ -t 1 ] && [ -n "$TERM" ] && [ "$TERM" != "dumb" ]; then
    # Optional: confirm terminal supports colors (tput)
    if command -v tput >/dev/null 2>&1 && tput colors >/dev/null 2>&1; then
        RED="\033[1;31m"
        YELLOW="\033[1;33m"
        CYAN="\033[1;36m"
        RESET="\033[0m"
    else
        RED=""; YELLOW=""; CYAN=""; RESET=""
    fi
else
    RED=""; YELLOW=""; CYAN=""; RESET=""
fi

# If first argument exists → use it
if [ $# -ge 1 ]; then
    INPUT="$1"
else
    # No argument → check if stdin is piped
    if [ ! -t 0 ]; then
        INPUT="/dev/stdin"
    else
      echo -e "${RED}Error:${RESET} no input file provided and nothing piped in." >&2
      echo -e "${YELLOW}Usage:${RESET} ${CYAN}./migrate.sh input.yaml${RESET}" >&2
      echo -e "${YELLOW}Usage:${RESET} ${CYAN}cat input.yaml | ./migrate.sh${RESET}" >&2
      exit 1
    fi
fi

INPUT="${1:-/dev/stdin}"

echo -e "${RED}WARNING:${RESET}"
echo -e "${YELLOW}  The input file *must* adhere to the OLD format:${RESET}"
echo -e "${YELLOW}      passwords:${RESET}"
echo -e "${YELLOW}      - key1: value${RESET}"
echo -e "${YELLOW}      - key2: value${RESET}"
echo -e "${YELLOW}  If it does NOT match this format, all passwords may be LOST.${RESET}"
echo
echo -e "${CYAN}TIP:${RESET} If you're not sure, make a copy of the file first:"
echo -e "${CYAN}     cp input.yaml input.backup.yaml${RESET}"
echo

echo -ne "${GREEN}Do you want to continue? Type 'Y' to proceed: ${RESET}"
read -r confirm

if [ "$confirm" != "Y" ]; then
    echo -e "${RED}Aborted by user.${RESET}"
    exit 1
fi

tmp=$(mktemp) || exit 1
trap 'rm -f -- "$tmp"' EXIT

yq -y '
  .passwords |= (
    map(
      to_entries[0] as $kv |
      {
        ($kv.key): {
          password: $kv.value,
          metadata: {date: null, text: null, url: null}
        }
      }
    ) | add
  )
' $INPUT > $tmp

mv -- "$tmp" "$INPUT"
trap - EXIT
