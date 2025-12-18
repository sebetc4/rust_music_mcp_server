#!/usr/bin/env python3
"""
STDIO transport test client for MCP server.

This script tests the STDIO transport layer by communicating with the
MCP server via stdin/stdout.

Usage:
    # Run tests (this starts the server automatically):
    python3 scripts/test_stdio_client.py
    
    # Or with cargo run directly:
    echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | cargo run
"""

import subprocess
import json
import sys
import time
from dataclasses import dataclass
from typing import Optional

# Configuration
SERVER_CMD = ["cargo", "run", "--release"]

@dataclass
class TestResult:
    """Result of a test case."""
    name: str
    passed: bool
    message: str
    duration_ms: float = 0.0

class McpStdioClient:
    """STDIO client for MCP server."""
    
    def __init__(self, cmd: list[str] = SERVER_CMD):
        self.cmd = cmd
        self.process = None
        self.request_id = 0
    
    def start(self):
        """Start the MCP server process."""
        self.process = subprocess.Popen(
            self.cmd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1
        )
        print(f"Started server: {' '.join(self.cmd)}")
    
    def stop(self):
        """Stop the MCP server process."""
        if self.process:
            self.process.terminate()
            self.process.wait(timeout=5)
            self.process = None
    
    def _next_id(self) -> int:
        """Get next request ID."""
        self.request_id += 1
        return self.request_id
    
    def send_request(self, method: str, params: Optional[dict] = None, timeout: float = 30.0) -> dict:
        """Send a JSON-RPC request and return the response."""
        request = {
            "jsonrpc": "2.0",
            "id": self._next_id(),
            "method": method,
        }
        if params is not None:
            request["params"] = params
        
        json_str = json.dumps(request)
        self.process.stdin.write(json_str + "\n")
        self.process.stdin.flush()
        
        # Read response with timeout
        import select
        start_time = time.time()
        response_line = ""
        
        while time.time() - start_time < timeout:
            # Check if there's data available
            ready, _, _ = select.select([self.process.stdout], [], [], 0.1)
            if ready:
                char = self.process.stdout.read(1)
                if char == '\n':
                    break
                response_line += char
        
        if not response_line:
            raise TimeoutError(f"No response received within {timeout}s")
        
        return json.loads(response_line)
    
    def send_notification(self, method: str, params: Optional[dict] = None):
        """Send a JSON-RPC notification (no response expected)."""
        notification = {
            "jsonrpc": "2.0",
            "method": method,
        }
        if params is not None:
            notification["params"] = params
        
        json_str = json.dumps(notification)
        self.process.stdin.write(json_str + "\n")
        self.process.stdin.flush()

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

def run_tests(client: McpStdioClient):
    """Run all STDIO transport tests."""
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
            "clientInfo": {"name": "stdio-test-client", "version": "1.0"}
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
        return passed, f"Found tools: {tools}" if passed else "Missing tools"
    
    results.append(run_test("List tools", test_list_tools))
    print_result(results[-1])
    
    # Test 3: List resources
    def test_list_resources():
        response = client.send_request("resources/list", {})
        passed = (
            "result" in response and
            "resources" in response["result"]
        )
        return passed, "Missing resources" if not passed else ""
    
    results.append(run_test("List resources", test_list_resources))
    print_result(results[-1])
    
    # Test 4: List prompts
    def test_list_prompts():
        response = client.send_request("prompts/list", {})
        passed = (
            "result" in response and
            "prompts" in response["result"]
        )
        return passed, "Missing prompts" if not passed else ""
    
    results.append(run_test("List prompts", test_list_prompts))
    print_result(results[-1])
    
    # =========================================================================
    # Filesystem Tools Tests
    # =========================================================================
    print_header("Filesystem Tools Tests")
    
    # Test 5: fs_list_dir
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
    
    # Test 6: mb_artist_search
    def test_mb_artist_search():
        response = client.send_request("tools/call", {
            "name": "mb_artist_search",
            "arguments": {
                "search_type": "artist",
                "query": "Daft Punk",
                "limit": 3
            }
        }, timeout=30)
        passed = (
            "result" in response and
            "content" in response["result"] and
            not response["result"].get("isError", False)
        )
        if passed:
            content = response["result"]["content"]
            if isinstance(content, list) and len(content) > 0:
                text = str(content[0])
                passed = "Daft Punk" in text or "Found" in text
        return passed, f"Got: {str(response.get('result', response.get('error')))[:200]}" if not passed else ""
    
    results.append(run_test("mb_artist_search (Daft Punk)", test_mb_artist_search))
    print_result(results[-1])
    
    # Rate limiting delay
    time.sleep(1.5)
    
    # Test 7: mb_release_search
    def test_mb_release_search():
        response = client.send_request("tools/call", {
            "name": "mb_release_search",
            "arguments": {
                "search_type": "release",
                "query": "Random Access Memories",
                "limit": 3
            }
        }, timeout=30)
        passed = (
            "result" in response and
            "content" in response["result"] and
            not response["result"].get("isError", False)
        )
        return passed, f"Got: {str(response.get('result', response.get('error')))[:200]}" if not passed else ""
    
    results.append(run_test("mb_release_search (Random Access Memories)", test_mb_release_search))
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
    print("  MCP Server STDIO Transport Test Suite")
    print("="*60)
    
    client = McpStdioClient()
    
    try:
        print("\nStarting server...")
        client.start()
        time.sleep(2)  # Wait for server to start
        
        success = run_tests(client)
        sys.exit(0 if success else 1)
    except KeyboardInterrupt:
        print("\nInterrupted by user")
        sys.exit(0)
    except Exception as e:
        print(f"\n❌ Error: {e}")
        sys.exit(1)
    finally:
        print("\nStopping server...")
        client.stop()

if __name__ == "__main__":
    main()
