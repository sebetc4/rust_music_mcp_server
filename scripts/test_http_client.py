#!/usr/bin/env python3
"""
HTTP transport test client for MCP server.

This script tests the HTTP transport layer by sending JSON-RPC requests
to the MCP server's HTTP endpoints.

Usage:
    # Start server first:
    cargo run --features http
    
    # Then run tests:
    python3 scripts/test_http_client.py
"""

import json
import requests
import sys
import time
from typing import Any, Optional
from dataclasses import dataclass

# Configuration
BASE_URL = "http://localhost:9090"
RPC_ENDPOINT = f"{BASE_URL}/mcp"
HEALTH_ENDPOINT = f"{BASE_URL}/health"

@dataclass
class TestResult:
    """Result of a test case."""
    name: str
    passed: bool
    message: str
    duration_ms: float = 0.0

class McpHttpClient:
    """Simple HTTP client for MCP server."""
    
    def __init__(self, base_url: str = BASE_URL):
        self.base_url = base_url
        self.rpc_url = f"{base_url}/mcp"
        self.request_id = 0
    
    def _next_id(self) -> int:
        """Get next request ID."""
        self.request_id += 1
        return self.request_id
    
    def send_rpc(self, method: str, params: Optional[dict] = None, timeout: int = 30) -> dict:
        """Send a JSON-RPC request and return the response."""
        request = {
            "jsonrpc": "2.0",
            "id": self._next_id(),
            "method": method,
        }
        if params:
            request["params"] = params
        
        response = requests.post(
            self.rpc_url,
            json=request,
            headers={"Content-Type": "application/json"},
            timeout=timeout
        )
        response.raise_for_status()
        return response.json()
    
    def health_check(self) -> dict:
        """Check server health."""
        response = requests.get(f"{self.base_url}/health", timeout=5)
        response.raise_for_status()
        return response.json()
    
    def get_info(self) -> dict:
        """Get server info from root endpoint."""
        response = requests.get(self.base_url, timeout=5)
        response.raise_for_status()
        return response.json()

def print_header(text: str):
    """Print a section header."""
    print(f"\n{'='*60}")
    print(f"  {text}")
    print('='*60)

def print_result(result: TestResult):
    """Print a test result."""
    status = "✅ PASS" if result.passed else "❌ FAIL"
    duration = f" ({result.duration_ms:.0f}ms)" if result.duration_ms > 0 else ""
    print(f"{status} | {result.name}{duration}")
    if not result.passed:
        print(f"       └── {result.message[:150]}")

def run_test(name: str, test_func) -> TestResult:
    """Run a test and return the result with timing."""
    start = time.time()
    try:
        passed, message = test_func()
        duration_ms = (time.time() - start) * 1000
        return TestResult(name, passed, message, duration_ms)
    except Exception as e:
        duration_ms = (time.time() - start) * 1000
        return TestResult(name, False, str(e), duration_ms)

def run_tests():
    """Run all HTTP transport tests."""
    client = McpHttpClient()
    results: list[TestResult] = []
    
    # =========================================================================
    # Basic Connectivity Tests
    # =========================================================================
    print_header("Basic Connectivity Tests")
    
    # Test 1: Health check
    def test_health():
        health = client.health_check()
        passed = health.get("status") == "healthy"
        return passed, f"Got: {health}" if not passed else ""
    
    results.append(run_test("Health check endpoint", test_health))
    print_result(results[-1])
    
    # Test 2: Root info endpoint
    def test_root_info():
        info = client.get_info()
        passed = info.get("protocol") == "JSON-RPC 2.0"
        return passed, f"Got: {info}" if not passed else ""
    
    results.append(run_test("Root info endpoint", test_root_info))
    print_result(results[-1])
    
    # =========================================================================
    # MCP Protocol Tests
    # =========================================================================
    print_header("MCP Protocol Tests")
    
    # Test 3: Initialize
    def test_initialize():
        response = client.send_rpc("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test-client", "version": "1.0"}
        })
        passed = (
            "result" in response and
            response["result"].get("protocolVersion") == "2024-11-05"
        )
        return passed, "Missing expected result" if not passed else ""
    
    results.append(run_test("Initialize session", test_initialize))
    print_result(results[-1])
    
    # Test 4: List tools
    def test_list_tools():
        response = client.send_rpc("tools/list")
        passed = (
            "result" in response and
            "tools" in response["result"] and
            len(response["result"]["tools"]) > 0
        )
        tools = [t["name"] for t in response.get("result", {}).get("tools", [])]
        return passed, f"Found tools: {tools}" if passed else "Missing tools in response"
    
    results.append(run_test("List tools", test_list_tools))
    print_result(results[-1])
    
    # Test 5: List resources
    def test_list_resources():
        response = client.send_rpc("resources/list")
        passed = (
            "result" in response and
            "resources" in response["result"]
        )
        resources = [r["uri"] for r in response.get("result", {}).get("resources", [])]
        return passed, f"Found {len(resources)} resources" if passed else "Missing resources in response"
    
    results.append(run_test("List resources", test_list_resources))
    print_result(results[-1])
    
    # Test 6: Read resource
    def test_read_resource():
        response = client.send_rpc("resources/read", {
            "uri": "mcp://server/info"
        })
        passed = "result" in response and "contents" in response["result"]
        return passed, f"Got error: {response.get('error')}" if not passed else ""
    
    results.append(run_test("Read resource (mcp://server/info)", test_read_resource))
    print_result(results[-1])
    
    # Test 7: List prompts
    def test_list_prompts():
        response = client.send_rpc("prompts/list")
        passed = (
            "result" in response and
            "prompts" in response["result"]
        )
        prompts = [p["name"] for p in response.get("result", {}).get("prompts", [])]
        return passed, f"Found prompts: {prompts}" if passed else "Missing prompts in response"
    
    results.append(run_test("List prompts", test_list_prompts))
    print_result(results[-1])
    
    # Test 8: Get prompt
    def test_get_prompt():
        response = client.send_rpc("prompts/get", {
            "name": "greeting",
            "arguments": {"name": "World"}
        })
        passed = "result" in response and "messages" in response["result"]
        return passed, f"Got error: {response.get('error')}" if not passed else ""
    
    results.append(run_test("Get prompt (greeting)", test_get_prompt))
    print_result(results[-1])
    
    # =========================================================================
    # Filesystem Tools Tests
    # =========================================================================
    print_header("Filesystem Tools Tests")
    
    # Test 9: fs_list_dir
    def test_fs_list_dir():
        response = client.send_rpc("tools/call", {
            "name": "fs_list_dir",
            "arguments": {"path": "."}
        })
        passed = (
            "result" in response and
            "content" in response["result"] and
            not response["result"].get("isError", False)
        )
        return passed, f"Got: {str(response.get('result', response.get('error')))[:100]}" if not passed else ""
    
    results.append(run_test("fs_list_dir tool", test_fs_list_dir))
    print_result(results[-1])
    
    # =========================================================================
    # Metadata Tools Tests
    # =========================================================================
    print_header("Metadata Tools Tests")
    
    # Test 10: read_metadata (non-existent file - should return error gracefully)
    def test_read_metadata_missing():
        response = client.send_rpc("tools/call", {
            "name": "read_metadata",
            "arguments": {"path": "/nonexistent/file.mp3"}
        })
        passed = "result" in response  # Should return result, possibly with isError=true
        return passed, f"Got: {response.get('error')}" if not passed else ""
    
    results.append(run_test("read_metadata (missing file)", test_read_metadata_missing))
    print_result(results[-1])
    
    # =========================================================================
    # MusicBrainz API Tools Tests
    # =========================================================================
    print_header("MusicBrainz API Tools Tests")
    print("  (These tests make real API calls and may take a few seconds)")
    
    # Test 11: mb_artist_search
    def test_mb_artist_search():
        response = client.send_rpc("tools/call", {
            "name": "mb_artist_search",
            "arguments": {
                "search_type": "artist",
                "query": "Nirvana",
                "limit": 3
            }
        }, timeout=30)
        passed = (
            "result" in response and
            "content" in response["result"] and
            not response["result"].get("isError", False)
        )
        # Check if result contains expected data
        if passed:
            content = response["result"]["content"]
            if isinstance(content, list) and len(content) > 0:
                text = str(content[0])
                passed = "Nirvana" in text or "Found" in text
        return passed, f"Got: {str(response.get('result', response.get('error')))[:200]}" if not passed else ""
    
    results.append(run_test("mb_artist_search (Nirvana)", test_mb_artist_search))
    print_result(results[-1])
    
    # Small delay to respect MusicBrainz rate limiting
    time.sleep(1.5)
    
    # Test 12: mb_release_search
    def test_mb_release_search():
        response = client.send_rpc("tools/call", {
            "name": "mb_release_search",
            "arguments": {
                "search_type": "release",
                "query": "Nevermind",
                "limit": 3
            }
        }, timeout=30)
        passed = (
            "result" in response and
            "content" in response["result"] and
            not response["result"].get("isError", False)
        )
        return passed, f"Got: {str(response.get('result', response.get('error')))[:200]}" if not passed else ""
    
    results.append(run_test("mb_release_search (Nevermind)", test_mb_release_search))
    print_result(results[-1])
    
    time.sleep(1.5)
    
    # Test 13: mb_recording_search
    def test_mb_recording_search():
        response = client.send_rpc("tools/call", {
            "name": "mb_recording_search",
            "arguments": {
                "search_type": "recording",
                "query": "Smells Like Teen Spirit",
                "limit": 3
            }
        }, timeout=30)
        passed = (
            "result" in response and
            "content" in response["result"] and
            not response["result"].get("isError", False)
        )
        return passed, f"Got: {str(response.get('result', response.get('error')))[:200]}" if not passed else ""
    
    results.append(run_test("mb_recording_search (Smells Like Teen Spirit)", test_mb_recording_search))
    print_result(results[-1])
    
    time.sleep(1.5)
    
    # Test 14: mb_advanced_search
    def test_mb_advanced_search():
        response = client.send_rpc("tools/call", {
            "name": "mb_advanced_search",
            "arguments": {
                "entity": "artist",
                "query": "Radiohead",
                "limit": 3
            }
        }, timeout=30)
        passed = (
            "result" in response and
            "content" in response["result"] and
            not response["result"].get("isError", False)
        )
        return passed, f"Got: {str(response.get('result', response.get('error')))[:200]}" if not passed else ""
    
    results.append(run_test("mb_advanced_search (Radiohead)", test_mb_advanced_search))
    print_result(results[-1])
    
    # =========================================================================
    # Error Handling Tests
    # =========================================================================
    print_header("Error Handling Tests")
    
    # Test 15: Unknown method
    def test_unknown_method():
        response = client.send_rpc("unknown/method")
        passed = (
            "error" in response and
            response["error"]["code"] == -32601  # Method not found
        )
        return passed, f"Expected -32601 error, got: {response}" if not passed else ""
    
    results.append(run_test("Unknown method returns error", test_unknown_method))
    print_result(results[-1])
    
    # Test 16: Invalid tool name
    def test_invalid_tool():
        response = client.send_rpc("tools/call", {
            "name": "nonexistent_tool",
            "arguments": {}
        })
        passed = "error" in response
        return passed, f"Expected error, got success: {response}" if not passed else ""
    
    results.append(run_test("Invalid tool returns error", test_invalid_tool))
    print_result(results[-1])
    
    # =========================================================================
    # Summary
    # =========================================================================
    print_header("Test Summary")
    
    passed = sum(1 for r in results if r.passed)
    failed = len(results) - passed
    total_time = sum(r.duration_ms for r in results)
    
    print(f"\n  Total:  {len(results)} tests")
    print(f"  Passed: {passed}")
    print(f"  Failed: {failed}")
    print(f"  Time:   {total_time:.0f}ms")
    
    if failed > 0:
        print("\n  Failed tests:")
        for r in results:
            if not r.passed:
                print(f"    - {r.name}: {r.message[:100]}")
    
    print()
    
    return failed == 0

def main():
    """Main entry point."""
    print("\n" + "="*60)
    print("  MCP Server HTTP Transport Test Suite")
    print("="*60)
    print(f"\nTarget: {BASE_URL}")
    print(f"RPC Endpoint: {RPC_ENDPOINT}")
    
    # Check if server is running
    try:
        requests.get(f"{BASE_URL}/health", timeout=2)
    except requests.exceptions.ConnectionError:
        print("\n❌ Error: Server not running!")
        print("\nStart the server first with:")
        print("  cargo run --features http")
        sys.exit(1)
    
    success = run_tests()
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
