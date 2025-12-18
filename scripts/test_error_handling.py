#!/usr/bin/env python3
"""
Test script to verify error handling in the MCP server.
Tests various error scenarios to ensure the server remains stable.
"""

import socket
import json
import sys
import time

def send_request(sock, method, params=None, request_id=1, expect_response=True):
    """Send a JSON-RPC request to the MCP server"""
    message = {
        "jsonrpc": "2.0",
        "id": request_id,
        "method": method,
    }
    if params is not None:
        message["params"] = params
    
    json_str = json.dumps(message)
    print(f"\n→ Sending: {json_str}")
    try:
        sock.sendall((json_str + "\n").encode('utf-8'))
        
        if not expect_response:
            print("(No response expected)")
            return None
            
        sock.settimeout(2.0)  # 2 second timeout
        response = sock.recv(4096).decode('utf-8')
        print(f"← Received: {response}")
        return json.loads(response) if response else None
    except socket.timeout:
        print(f"✗ Timeout waiting for response")
        return None
    except Exception as e:
        print(f"✗ Error: {e}")
        return None

def test_error_scenario(name, test_fn):
    """Run a single error test scenario"""
    print("\n" + "="*60)
    print(f"TEST: {name}")
    print("="*60)
    try:
        test_fn()
        print(f"✓ Test '{name}' completed")
    except Exception as e:
        print(f"✗ Test '{name}' failed: {e}")

def main():
    host = "127.0.0.1"
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 4000
    
    print(f"Testing MCP server error handling at {host}:{port}")
    
    # Test 1: Missing required parameters
    def test_missing_params():
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.connect((host, port))
            send_request(sock, "initialize", {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0.0"}
            }, 1)
            notification = {"jsonrpc": "2.0", "method": "notifications/initialized"}
            sock.sendall((json.dumps(notification) + "\n").encode('utf-8'))
            time.sleep(0.1)
            
            # Call tool with missing parameters
            send_request(sock, "tools/call", {
                "name": "echo"
                # Missing "arguments" field
            }, 2)
    
    test_error_scenario("Missing required parameters", test_missing_params)
    
    # Test 2: Invalid tool name
    def test_invalid_tool():
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.connect((host, port))
            send_request(sock, "initialize", {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0.0"}
            }, 1)
            notification = {"jsonrpc": "2.0", "method": "notifications/initialized"}
            sock.sendall((json.dumps(notification) + "\n").encode('utf-8'))
            time.sleep(0.1)
            
            send_request(sock, "tools/call", {
                "name": "nonexistent_tool",
                "arguments": {}
            }, 2)
    
    test_error_scenario("Invalid tool name", test_invalid_tool)
    
    # Test 3: Invalid method (treated as notification, no response expected)
    def test_invalid_method():
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.connect((host, port))
            send_request(sock, "initialize", {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0.0"}
            }, 1)
            notification = {"jsonrpc": "2.0", "method": "notifications/initialized"}
            sock.sendall((json.dumps(notification) + "\n").encode('utf-8'))
            time.sleep(0.1)
            
            # Invalid methods with IDs should get error responses
            result = send_request(sock, "invalid/method", {}, 2)
            if result and "error" in result:
                print("✓ Received error response for invalid method")
            else:
                print("Note: No error response (may be treated as notification)")
    
    test_error_scenario("Invalid method", test_invalid_method)
    
    # Test 4: Malformed JSON
    def test_malformed_json():
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.connect((host, port))
            send_request(sock, "initialize", {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0.0"}
            }, 1)
            notification = {"jsonrpc": "2.0", "method": "notifications/initialized"}
            sock.sendall((json.dumps(notification) + "\n").encode('utf-8'))
            time.sleep(0.1)
            
            # Send malformed JSON
            print("\n→ Sending: {invalid json}")
            sock.sendall(b"{invalid json}\n")
            time.sleep(0.5)
    
    test_error_scenario("Malformed JSON", test_malformed_json)
    
    # Test 5: Wrong parameter types
    def test_wrong_types():
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.connect((host, port))
            send_request(sock, "initialize", {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0.0"}
            }, 1)
            notification = {"jsonrpc": "2.0", "method": "notifications/initialized"}
            sock.sendall((json.dumps(notification) + "\n").encode('utf-8'))
            time.sleep(0.1)
            
            # Call add tool with string instead of numbers
            send_request(sock, "tools/call", {
                "name": "add",
                "arguments": {
                    "a": "not_a_number",
                    "b": "also_not_a_number"
                }
            }, 2)
    
    test_error_scenario("Wrong parameter types", test_wrong_types)
    
    # Test 6: Connection survives after errors
    def test_connection_survival():
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.connect((host, port))
            send_request(sock, "initialize", {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0.0"}
            }, 1)
            notification = {"jsonrpc": "2.0", "method": "notifications/initialized"}
            sock.sendall((json.dumps(notification) + "\n").encode('utf-8'))
            time.sleep(0.1)
            
            # Send an invalid request
            send_request(sock, "tools/call", {"name": "invalid"}, 2)
            
            # Try a valid request after the error
            time.sleep(0.2)
            result = send_request(sock, "tools/call", {
                "name": "echo",
                "arguments": {"message": "Still working!"}
            }, 3)
            
            if result and "result" in result:
                print("✓ Connection survived the error!")
            else:
                print("✗ Connection did not survive")
    
    test_error_scenario("Connection survival after error", test_connection_survival)
    
    # Test 7: Server accepts new connections after errors
    def test_new_connection_after_errors():
        # First connection with errors
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.connect((host, port))
            send_request(sock, "initialize", {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test1", "version": "1.0.0"}
            }, 1)
            # Don't send initialized notification - this will cause errors
            send_request(sock, "tools/list", {}, 2)
        
        time.sleep(0.5)
        
        # New connection should work fine
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
            sock.connect((host, port))
            result = send_request(sock, "initialize", {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test2", "version": "1.0.0"}
            }, 1)
            
            if result and "result" in result:
                print("✓ Server accepted new connection after previous errors!")
            else:
                print("✗ Server did not accept new connection")
    
    test_error_scenario("New connection after errors", test_new_connection_after_errors)
    
    print("\n" + "="*60)
    print("Error handling tests completed!")
    print("Check server logs to verify errors were handled gracefully")
    print("="*60)

if __name__ == "__main__":
    try:
        main()
    except ConnectionRefusedError:
        print("Error: Could not connect to server. Is it running?")
        print("Start the server with: MCP_TRANSPORT=tcp MCP_TCP_PORT=4000 cargo run")
        sys.exit(1)
    except KeyboardInterrupt:
        print("\nInterrupted by user")
        sys.exit(0)
