#!/usr/bin/env python3
"""
TCP transport test client for MCP server.

This script tests the TCP transport layer by sending JSON-RPC messages
over a raw TCP socket.

Usage:
    # Start server first:
    cargo run --features tcp
    
    # Then run tests:
    python3 scripts/test_tcp_client.py [port]
"""

import socket
import json
import sys
import time
from dataclasses import dataclass
from typing import Optional

# Configuration
DEFAULT_HOST = "127.0.0.1"
DEFAULT_PORT = 4000

@dataclass
class TestResult:
    """Result of a test case."""
    name: str
    passed: bool
    message: str
    duration_ms: float = 0.0

class McpTcpClient:
    """Simple TCP client for MCP server."""
    
    def __init__(self, host: str = DEFAULT_HOST, port: int = DEFAULT_PORT):
        self.host = host
        self.port = port
        self.sock = None
        self.request_id = 0
        self.buffer = ""
    
    def connect(self):
        """Connect to the MCP server."""
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.sock.settimeout(30)  # 30 second timeout for MusicBrainz calls
        self.sock.connect((self.host, self.port))
        print(f"Connected to {self.host}:{self.port}")
    
    def close(self):
        """Close the connection."""
        if self.sock:
            self.sock.close()
            self.sock = None
    
    def _next_id(self) -> int:
        """Get next request ID."""
        self.request_id += 1
        return self.request_id
    
    def send_request(self, method: str, params: Optional[dict] = None) -> dict:
        """Send a JSON-RPC request and return the response."""
        request = {
            "jsonrpc": "2.0",
            "id": self._next_id(),
            "method": method,
        }
        if params is not None:
            request["params"] = params
        
        json_str = json.dumps(request)
        self.sock.sendall((json_str + "\n").encode('utf-8'))
        
        # Receive response (may come in chunks)
        while "\n" not in self.buffer:
            chunk = self.sock.recv(8192).decode('utf-8')
            if not chunk:
                raise ConnectionError("Connection closed by server")
            self.buffer += chunk
        
        # Extract one line
        line, self.buffer = self.buffer.split("\n", 1)
        return json.loads(line)
    
    def send_notification(self, method: str, params: Optional[dict] = None):
        """Send a JSON-RPC notification (no response expected)."""
        notification = {
            "jsonrpc": "2.0",
            "method": method,
        }
        if params is not None:
            notification["params"] = params
        
        json_str = json.dumps(notification)
        self.sock.sendall((json_str + "\n").encode('utf-8'))

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

def run_tests(client: McpTcpClient):
    """Run all TCP transport tests."""
    results: list[TestResult] = []
    
    # =========================================================================
    # MCP Protocol Tests
    # =========================================================================
    print_header("MCP Protocol Tests")
    
    # Test 1: Initialize
    def test_initialize():
        response = client.send_request("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "tcp-test-client", "version": "1.0"}
        })
        passed = (
            "result" in response and
            response["result"].get("protocolVersion") == "2024-11-05"
        )
        return passed, "Missing expected result" if not passed else ""
    
    results.append(run_test("Initialize session", test_initialize))
    print_result(results[-1])
    
    # Send initialized notification
    client.send_notification("notifications/initialized")
    print("  ↳ Sent initialized notification")
    
    # Test 2: List tools
    def test_list_tools():
        response = client.send_request("tools/list", {})
        passed = (
            "result" in response and
            "tools" in response["result"] and
            len(response["result"]["tools"]) > 0
        )
        tools = [t["name"] for t in response.get("result", {}).get("tools", [])]
        return passed, f"Found tools: {tools}" if passed else "Missing tools in response"
    
    results.append(run_test("List tools", test_list_tools))
    print_result(results[-1])
    
    # Test 3: List resources
    def test_list_resources():
        response = client.send_request("resources/list", {})
        passed = (
            "result" in response and
            "resources" in response["result"]
        )
        resources = [r["uri"] for r in response.get("result", {}).get("resources", [])]
        return passed, f"Found {len(resources)} resources" if passed else "Missing resources"
    
    results.append(run_test("List resources", test_list_resources))
    print_result(results[-1])
    
    # Test 4: Read resource
    def test_read_resource():
        response = client.send_request("resources/read", {
            "uri": "mcp://server/info"
        })
        passed = "result" in response and "contents" in response["result"]
        return passed, f"Got error: {response.get('error')}" if not passed else ""
    
    results.append(run_test("Read resource (mcp://server/info)", test_read_resource))
    print_result(results[-1])
    
    # Test 5: List prompts
    def test_list_prompts():
        response = client.send_request("prompts/list", {})
        passed = (
            "result" in response and
            "prompts" in response["result"]
        )
        prompts = [p["name"] for p in response.get("result", {}).get("prompts", [])]
        return passed, f"Found prompts: {prompts}" if passed else "Missing prompts"
    
    results.append(run_test("List prompts", test_list_prompts))
    print_result(results[-1])
    
    # Test 6: Get prompt
    def test_get_prompt():
        response = client.send_request("prompts/get", {
            "name": "greeting",
            "arguments": {"name": "TCP Client"}
        })
        passed = "result" in response and "messages" in response["result"]
        return passed, f"Got error: {response.get('error')}" if not passed else ""
    
    results.append(run_test("Get prompt (greeting)", test_get_prompt))
    print_result(results[-1])
    
    # =========================================================================
    # Filesystem Tools Tests
    # =========================================================================
    print_header("Filesystem Tools Tests")
    
    # Test 7: fs_list_dir
    def test_fs_list_dir():
        response = client.send_request("tools/call", {
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
    # MusicBrainz API Tools Tests
    # =========================================================================
    print_header("MusicBrainz API Tools Tests")
    print("  (These tests make real API calls and may take a few seconds)")
    
    # Test 8: mb_artist_search
    def test_mb_artist_search():
        response = client.send_request("tools/call", {
            "name": "mb_artist_search",
            "arguments": {
                "search_type": "artist",
                "query": "Nirvana",
                "limit": 3
            }
        })
        passed = (
            "result" in response and
            "content" in response["result"] and
            not response["result"].get("isError", False)
        )
        if passed:
            content = response["result"]["content"]
            if isinstance(content, list) and len(content) > 0:
                text = str(content[0])
                passed = "Nirvana" in text or "Found" in text
        return passed, f"Got: {str(response.get('result', response.get('error')))[:200]}" if not passed else ""
    
    results.append(run_test("mb_artist_search (Nirvana)", test_mb_artist_search))
    print_result(results[-1])
    
    # Rate limiting delay
    time.sleep(1.5)
    
    # Test 9: mb_release_search
    def test_mb_release_search():
        response = client.send_request("tools/call", {
            "name": "mb_release_search",
            "arguments": {
                "search_type": "release",
                "query": "OK Computer",
                "limit": 3
            }
        })
        passed = (
            "result" in response and
            "content" in response["result"] and
            not response["result"].get("isError", False)
        )
        return passed, f"Got: {str(response.get('result', response.get('error')))[:200]}" if not passed else ""
    
    results.append(run_test("mb_release_search (OK Computer)", test_mb_release_search))
    print_result(results[-1])
    
    time.sleep(1.5)
    
    # Test 10: mb_recording_search
    def test_mb_recording_search():
        response = client.send_request("tools/call", {
            "name": "mb_recording_search",
            "arguments": {
                "search_type": "recording",
                "query": "Paranoid Android",
                "limit": 3
            }
        })
        passed = (
            "result" in response and
            "content" in response["result"] and
            not response["result"].get("isError", False)
        )
        return passed, f"Got: {str(response.get('result', response.get('error')))[:200]}" if not passed else ""
    
    results.append(run_test("mb_recording_search (Paranoid Android)", test_mb_recording_search))
    print_result(results[-1])
    
    time.sleep(1.5)
    
    # Test 11: mb_advanced_search
    def test_mb_advanced_search():
        response = client.send_request("tools/call", {
            "name": "mb_advanced_search",
            "arguments": {
                "entity": "label",
                "query": "Sony",
                "limit": 3
            }
        })
        passed = (
            "result" in response and
            "content" in response["result"] and
            not response["result"].get("isError", False)
        )
        return passed, f"Got: {str(response.get('result', response.get('error')))[:200]}" if not passed else ""
    
    results.append(run_test("mb_advanced_search (label: Sony)", test_mb_advanced_search))
    print_result(results[-1])
    
    # =========================================================================
    # Error Handling Tests
    # =========================================================================
    print_header("Error Handling Tests")
    
    # Test 12: Invalid tool
    def test_invalid_tool():
        response = client.send_request("tools/call", {
            "name": "nonexistent_tool",
            "arguments": {}
        })
        passed = "error" in response
        return passed, f"Expected error, got: {response}" if not passed else ""
    
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
    host = DEFAULT_HOST
    port = int(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_PORT
    
    print("\n" + "="*60)
    print("  MCP Server TCP Transport Test Suite")
    print("="*60)
    print(f"\nTarget: {host}:{port}")
    
    client = McpTcpClient(host, port)
    
    try:
        client.connect()
        success = run_tests(client)
        sys.exit(0 if success else 1)
    except ConnectionRefusedError:
        print("\n❌ Error: Could not connect to server!")
        print("\nStart the server first with:")
        print(f"  cargo run --features tcp")
        sys.exit(1)
    except KeyboardInterrupt:
        print("\nInterrupted by user")
        sys.exit(0)
    finally:
        client.close()

if __name__ == "__main__":
    main()
