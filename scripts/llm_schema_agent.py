#!/usr/bin/env python3
"""
scripts/llm_schema_agent.py

A small example "agent" script that asks an OpenAI-compatible LLM to generate a pw schema
(JSON or TOML), validates it against the minimal shape expected by pw (see src/schema.rs),
and writes the result to files.

Usage (quick):
  pip install -r scripts/requirements.txt
  python scripts/llm_schema_agent.py --cmd ssh --prompt "Generate a schema for ssh that helps set host, user, and port"

The script looks for an OpenAI API key in OPENAI_API_KEY and an optional base URL in OPENAI_API_BASE.
If no API key is present, you may pass LLM output via --stdin or paste it when prompted.

This is a demonstration; the model prompt and validation are intentionally simple. Review output
before using with --dryrun disabled.
"""

from __future__ import annotations
import argparse
import json
import os
import sys
import textwrap
from typing import Any, Dict, List, Optional, Tuple

try:
    import requests
except Exception:
    print("Please install the 'requests' package: pip install requests", file=sys.stderr)
    sys.exit(2)

try:
    import toml
except Exception:
    print("Please install the 'toml' package: pip install toml", file=sys.stderr)
    sys.exit(2)

ALLOWED_KINDS = {"select", "filepath", "str", "number", "flag", "pairs"}

EXAMPLE_SCHEMA = {
    "exe": "ssh",
    "description": "Connect to a remote host",
    "args": [
        {"name": "host", "kind": {"kind": "str"}, "required": True, "description": "Remote host"},
        {"name": "port", "kind": {"kind": "number", "int": True, "min": 1, "max": 65535}, "description": "SSH port"},
        {"name": "user", "kind": {"kind": "str"}, "description": "Username"},
        {"name": "identity", "kind": {"kind": "filepath", "dir_only": False}, "description": "Identity file"},
    ],
}

PROMPT_TEMPLATE = textwrap.dedent(
    """
    You are an assistant that produces a schema for the `pw` tool (Prompt Wrapper).
    The expected output is either valid JSON or valid TOML that deserializes into the
    following shape (give only the object, do not surround by any other text):

    - top-level keys:
      - exe (string)  -- executable name
      - description (string, optional)
      - args (array of argument objects, optional)

    - argument object:
      - name (string)
      - kind (object) with a field `kind` whose value is one of: select, filepath, str, number, flag, pairs
        and optional kind-specific keys (e.g., options for select, int/min/max for number)
      - required (bool, optional)
      - repeatable (bool, optional)
      - description / desc (string, optional)
      - conflicts_with / depends_on (array of strings, optional)

    Produce the schema for the command: {cmd}

    Use the examples below as guidance (JSON shown):
    {example_json}

    Output only the schema in JSON or TOML format. Do not include explanation text.
    """
)


def build_prompt(cmd: str, help_text: Optional[str] = None) -> str:
    example_json = json.dumps(EXAMPLE_SCHEMA, indent=2)
    prompt = PROMPT_TEMPLATE.format(cmd=cmd, example_json=example_json)
    if help_text:
        prompt += "\nAdditional context (help output):\n" + help_text
    return prompt


def call_openai_chat(prompt: str, model: str = "gpt-4") -> str:
    api_key = os.environ.get("OPENAI_API_KEY")
    if not api_key:
        raise RuntimeError("OPENAI_API_KEY not set; cannot call the LLM endpoint")
    api_base = os.environ.get("OPENAI_API_BASE", "https://api.openai.com/v1")
    url = api_base.rstrip("/") + "/chat/completions"
    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json",
    }
    payload = {
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": 1000,
        "temperature": 0.0,
    }
    resp = requests.post(url, headers=headers, json=payload, timeout=60)
    resp.raise_for_status()
    out = resp.json()
    # The exact shape depends on the API; try common locations
    if "choices" in out and len(out["choices"]) > 0:
        text = out["choices"][0].get("message", {}).get("content") or out["choices"][0].get("text")
        return text
    # fallback
    if isinstance(out, dict):
        return json.dumps(out)
    return str(out)


def parse_llm_output(text: str) -> Tuple[Optional[Dict[str, Any]], Optional[str]]:
    """Attempt to parse LLM output as JSON first, then TOML. Return (dict, format)"""
    text_stripped = text.strip()
    # Try to extract code block if present
    if text_stripped.startswith("```"):
        # remove triple backticks and optional language spec
        parts = text_stripped.split("\n")
        # find first line with ``` and remove it
        if parts:
            # remove first and last fence
            try:
                first_line = parts[0]
                if first_line.startswith("```"):
                    parts = parts[1:]
                if parts and parts[-1].startswith("```"):
                    parts = parts[:-1]
                text_stripped = "\n".join(parts).strip()
            except Exception:
                pass
    # Try JSON
    try:
        parsed = json.loads(text_stripped)
        if isinstance(parsed, dict):
            return parsed, "json"
    except Exception:
        pass
    # Try TOML
    try:
        parsed = toml.loads(text_stripped)
        if isinstance(parsed, dict):
            return parsed, "toml"
    except Exception:
        pass
    return None, None


def validate_schema(obj: Dict[str, Any]) -> List[str]:
    errors: List[str] = []
    if "exe" not in obj or not isinstance(obj.get("exe"), str):
        errors.append("'exe' is required and must be a string")
    if "args" in obj:
        if not isinstance(obj["args"], list):
            errors.append("'args' must be an array if present")
        else:
            for i, a in enumerate(obj["args"]):
                if not isinstance(a, dict):
                    errors.append(f"args[{i}] must be an object")
                    continue
                if "name" not in a or not isinstance(a.get("name"), str):
                    errors.append(f"args[{i}].name is required and must be a string")
                kind = a.get("kind")
                if not isinstance(kind, dict) or "kind" not in kind:
                    errors.append(f"args[{i}].kind is required and must be an object with a 'kind' field")
                else:
                    k = str(kind.get("kind")).lower()
                    if k not in ALLOWED_KINDS:
                        errors.append(f"args[{i}].kind.kind must be one of {sorted(ALLOWED_KINDS)} (got '{k}')")
    return errors


def write_outputs(obj: Dict[str, Any], out_basename: str) -> Tuple[str, str]:
    json_path = out_basename + ".json"
    toml_path = out_basename + ".toml"
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump(obj, f, indent=2, ensure_ascii=False)
    with open(toml_path, "w", encoding="utf-8") as f:
        toml_str = toml.dumps(obj)
        f.write(toml_str)
    return json_path, toml_path


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(description="LLM-driven schema generator for pw (demo)")
    parser.add_argument("--cmd", required=True, help="The command to generate a schema for (e.g., ssh, docker)")
    parser.add_argument("--help-output", help="Optional help text / context to include for the LLM")
    parser.add_argument("--model", default="gpt-4", help="LLM model name (for OpenAI-compatible API)")
    parser.add_argument("--out", default="generated_schema", help="Output basename (no extension)")
    parser.add_argument("--dryrun", action="store_true", help="Don't write files; just print parsed schema")
    parser.add_argument("--stdin", action="store_true", help="Read LLM output from stdin instead of calling an API")
    args = parser.parse_args(argv)

    prompt = build_prompt(args.cmd, args.help_output)

    raw_text: Optional[str] = None
    try:
        if args.stdin:
            print("Reading LLM output from stdin; paste and finish with EOF (Ctrl-D)")
            raw_text = sys.stdin.read()
        else:
            # call LLM
            print("Calling LLM... (set OPENAI_API_KEY to use) ")
            raw_text = call_openai_chat(prompt, model=args.model)
    except Exception as e:
        print(f"LLM call failed: {e}")
        print("You can re-run with --stdin and paste model output.")
        return 3

    if not raw_text:
        print("No output from LLM")
        return 4

    parsed, fmt = parse_llm_output(raw_text)
    if parsed is None:
        print("Failed to parse LLM output as JSON or TOML. Raw output below:\n")
        print(raw_text)
        return 5

    errors = validate_schema(parsed)
    if errors:
        print("Schema validation failed with the following errors:")
        for e in errors:
            print(" - ", e)
        print("\nRaw parsed object:\n")
        print(json.dumps(parsed, indent=2, ensure_ascii=False))
        return 6

    print("Schema validated OK (minimal checks). Parsed as:", fmt)
    if args.dryrun:
        print(json.dumps(parsed, indent=2, ensure_ascii=False))
        return 0

    out_json, out_toml = write_outputs(parsed, args.out)
    print(f"Wrote: {out_json} and {out_toml}")
    print("Tip: drop the TOML/JSON into one of your pw config dirs or run pw with -C pointing to the directory")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
