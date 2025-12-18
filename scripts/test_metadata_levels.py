#!/usr/bin/env python3
"""
Test script to verify metadata_level parameter works correctly.

This script tests that the mb_identify_record tool accepts all three
metadata levels: minimal, basic, and full.

Usage:
    # Start server first:
    cargo run --features http

    # Then run test:
    python3 scripts/test_metadata_levels.py
"""

import json
import requests
import sys

BASE_URL = "http://localhost:4000/mcp"

def test_metadata_level(level: str) -> bool:
    """Test that a specific metadata level is accepted."""
    payload = {
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "mb_identify_record",
            "arguments": {
                "file_path": "/nonexistent/test.mp3",  # Will fail fast at file check
                "metadata_level": level
            }
        },
        "id": 1
    }

    try:
        response = requests.post(BASE_URL, json=payload, timeout=5)
        response.raise_for_status()
        result = response.json()

        # We expect the call to fail due to missing file, but it should NOT
        # fail due to invalid parameter
        if "result" in result:
            # Check that error is about file, not parameter
            content = result["result"].get("content", [])
            if content and isinstance(content, list):
                text = content[0].get("text", "")
                if "File not found" in text or "not found" in text.lower():
                    print(f"✅ PASS: metadata_level='{level}' - Parameter accepted (failed at file check as expected)")
                    return True
                else:
                    print(f"❌ FAIL: metadata_level='{level}' - Unexpected error: {text[:100]}")
                    return False

        if "error" in result:
            error_msg = result["error"].get("message", "")
            if "metadata_level" in error_msg.lower() or "parameter" in error_msg.lower():
                print(f"❌ FAIL: metadata_level='{level}' - Parameter rejected: {error_msg}")
                return False
            else:
                print(f"✅ PASS: metadata_level='{level}' - Parameter accepted (other error occurred)")
                return True

        print(f"❓ UNKNOWN: metadata_level='{level}' - Unexpected response: {result}")
        return False

    except Exception as e:
        print(f"❌ ERROR: metadata_level='{level}' - Request failed: {e}")
        return False

def test_schema_availability():
    """Test that the schema includes metadata_level field."""
    payload = {
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 0
    }

    try:
        response = requests.post(BASE_URL, json=payload, timeout=5)
        response.raise_for_status()
        result = response.json()

        if "result" not in result or "tools" not in result["result"]:
            print("❌ FAIL: Could not list tools")
            return False

        # Find mb_identify_record tool
        tools = result["result"]["tools"]
        identify_tool = next((t for t in tools if t["name"] == "mb_identify_record"), None)

        if not identify_tool:
            print("❌ FAIL: mb_identify_record tool not found")
            return False

        # Check schema
        schema = identify_tool.get("inputSchema", {})
        properties = schema.get("properties", {})

        if "metadata_level" not in properties:
            print("❌ FAIL: metadata_level not in schema properties")
            return False

        metadata_level_schema = properties["metadata_level"]

        # Check that it has the enum definition
        if "$ref" in metadata_level_schema:
            # It's a reference, check definitions
            defs = schema.get("$defs", {})
            if "MetadataLevel" in defs:
                metadata_def = defs["MetadataLevel"]
                if "oneOf" in metadata_def:
                    values = [opt.get("const") for opt in metadata_def["oneOf"]]
                    expected = ["minimal", "basic", "full"]
                    if set(values) == set(expected):
                        print(f"✅ PASS: Schema includes metadata_level with values: {values}")
                        return True
                    else:
                        print(f"❌ FAIL: Schema has unexpected values: {values}")
                        return False

        print(f"❓ UNKNOWN: metadata_level schema format unexpected: {metadata_level_schema}")
        return False

    except Exception as e:
        print(f"❌ ERROR: Schema check failed: {e}")
        return False

def main():
    print("="*60)
    print("  Metadata Level Parameter Test")
    print("="*60)
    print()

    # Check server is running
    try:
        requests.get("http://localhost:4000/health", timeout=2)
    except:
        print("❌ Error: Server not running on port 4000!")
        print("\nStart the server first with:")
        print("  cargo run --features http")
        sys.exit(1)

    print("Testing schema availability...")
    schema_ok = test_schema_availability()
    print()

    if not schema_ok:
        print("⚠️  Schema test failed, but continuing with parameter tests...\n")

    print("Testing all metadata levels...")
    print()

    results = {
        "minimal": test_metadata_level("minimal"),
        "basic": test_metadata_level("basic"),
        "full": test_metadata_level("full"),
    }

    print()
    print("="*60)
    print("  Summary")
    print("="*60)
    passed = sum(1 for v in results.values() if v)
    total = len(results)
    print(f"\nSchema check: {'✅ PASS' if schema_ok else '❌ FAIL'}")
    print(f"Parameter tests: {passed}/{total} passed")

    if all(results.values()) and schema_ok:
        print("\n✅ All tests passed! The metadata_level parameter is working correctly.")
        print("\nYou can now use it in Insomnia with values:")
        print('  - "minimal"')
        print('  - "basic"')
        print('  - "full"')
        sys.exit(0)
    else:
        print("\n❌ Some tests failed.")
        sys.exit(1)

if __name__ == "__main__":
    main()
