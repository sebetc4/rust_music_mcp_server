#!/usr/bin/env python3
"""
Test script to identify a real audio file with different metadata levels.

This script tests the mb_identify_record tool with an actual audio file
to demonstrate the differences between minimal, basic, and full metadata levels.

Usage:
    # Start server first:
    cargo run --features http

    # Then run test:
    python3 scripts/test_identify_real.py
"""

import json
import requests
import sys

BASE_URL = "http://localhost:4000/mcp"
TEST_FILE = "/mnt/code/MCP/music_mcp_server/test/test.mp3"

def test_identify_with_level(level: str) -> dict:
    """Test identification with a specific metadata level."""
    payload = {
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "mb_identify_record",
            "arguments": {
                "file_path": TEST_FILE,
                "metadata_level": level,
                "limit": 2
            }
        },
        "id": 1
    }

    try:
        print(f"\n{'='*70}")
        print(f"Testing metadata_level: {level}")
        print(f"{'='*70}\n")

        response = requests.post(BASE_URL, json=payload, timeout=60)
        response.raise_for_status()
        result = response.json()

        if "result" in result:
            content = result["result"].get("content", [])
            if content and isinstance(content, list):
                text = content[0].get("text", "")
                is_error = result["result"].get("isError", False)

                if is_error:
                    print(f"❌ ERROR: {text[:200]}...")
                    return {"level": level, "success": False, "error": text}
                else:
                    print(text)
                    return {"level": level, "success": True, "output": text}

        if "error" in result:
            error_msg = result["error"].get("message", "Unknown error")
            print(f"❌ RPC ERROR: {error_msg}")
            return {"level": level, "success": False, "error": error_msg}

        print(f"❓ UNEXPECTED RESPONSE: {result}")
        return {"level": level, "success": False, "error": "Unexpected response"}

    except Exception as e:
        print(f"❌ REQUEST FAILED: {e}")
        return {"level": level, "success": False, "error": str(e)}

def main():
    print("="*70)
    print("  MusicBrainz Audio Identification - Real File Test")
    print("="*70)
    print(f"\nTest file: {TEST_FILE}")
    print()

    # Check server is running
    try:
        requests.get("http://localhost:4000/health", timeout=2)
        print("✅ Server is running on port 4000\n")
    except:
        print("❌ Error: Server not running on port 4000!")
        print("\nStart the server first with:")
        print("  cargo run --features http")
        sys.exit(1)

    # Test all three levels
    results = []
    for level in ["minimal", "basic", "full"]:
        result = test_identify_with_level(level)
        results.append(result)

        # Wait a bit between requests to avoid rate limiting
        if level != "full":
            print("\nWaiting 2 seconds before next request...")
            import time
            time.sleep(2)

    # Summary
    print("\n" + "="*70)
    print("  Summary")
    print("="*70)

    successful = sum(1 for r in results if r["success"])
    print(f"\nSuccessful requests: {successful}/3")

    for result in results:
        status = "✅" if result["success"] else "❌"
        print(f"{status} {result['level']}")

    if successful == 3:
        print("\n✅ All identification requests completed successfully!")
        print("\nYou can now use metadata_level in Insomnia:")
        print('  1. In the request body, add "metadata_level": "minimal"')
        print('  2. Change to "basic" or "full" for more details')
        sys.exit(0)
    else:
        print("\n⚠️  Some requests failed. This might be due to:")
        print("  - Invalid AcoustID API key")
        print("  - Network issues")
        print("  - Rate limiting")
        print("  - Audio file not in AcoustID database")
        sys.exit(1)

if __name__ == "__main__":
    main()
