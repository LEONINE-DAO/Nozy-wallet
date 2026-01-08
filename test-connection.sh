#!/bin/bash

# NozyWallet Connection Test Script
# Tests connectivity to zec.leoninedao.org:443

set -e

echo "=== NozyWallet Connection Test ==="
echo ""

# Configuration
HOST="zec.leoninedao.org"
PORT="443"
IP="45.55.79.71"

echo "1. Testing DNS resolution:"
if host $HOST > /dev/null 2>&1; then
    echo "✅ DNS resolution successful"
    host $HOST | head -1
else
    echo "❌ DNS resolution failed"
    exit 1
fi
echo ""

echo "2. Testing port connectivity (TCP):"
if timeout 5 bash -c "echo > /dev/tcp/$HOST/$PORT" 2>/dev/null; then
    echo "✅ Port $PORT is reachable on $HOST"
else
    echo "❌ Port $PORT not reachable on $HOST"
    echo "   This could mean:"
    echo "   - Firewall is blocking"
    echo "   - Service is not running"
    echo "   - Network issue"
fi
echo ""

echo "3. Testing with IP address:"
if timeout 5 bash -c "echo > /dev/tcp/$IP/$PORT" 2>/dev/null; then
    echo "✅ Port $PORT is reachable on IP $IP"
else
    echo "❌ Port $PORT not reachable on IP $IP"
fi
echo ""

echo "4. Testing HTTPS connection:"
if curl -k -v --connect-timeout 10 --max-time 30 https://$HOST:$PORT 2>&1 | grep -q "HTTP"; then
    echo "✅ HTTPS connection successful"
    echo "   Response headers:"
    curl -k -I --connect-timeout 10 --max-time 30 https://$HOST:$PORT 2>&1 | head -10
else
    echo "❌ HTTPS connection failed"
    echo "   Full response:"
    curl -k -v --connect-timeout 10 --max-time 30 https://$HOST:$PORT 2>&1 | head -20
fi
echo ""

echo "5. Checking firewall rules:"
if command -v ufw > /dev/null 2>&1; then
    echo "UFW status:"
    ufw status verbose 2>/dev/null || echo "UFW not active"
elif command -v iptables > /dev/null 2>&1; then
    echo "Checking iptables for port $PORT:"
    iptables -L -n | grep $PORT || echo "No specific rule for port $PORT"
else
    echo "No firewall tool found (ufw/iptables)"
fi
echo ""

echo "6. Testing from external perspective:"
echo "   Simulating external connection test..."
if timeout 5 bash -c "echo > /dev/tcp/$IP/$PORT" 2>/dev/null; then
    echo "✅ Port $PORT is reachable externally on $IP"
else
    echo "❌ Port $PORT not reachable externally"
    echo "   Possible issues:"
    echo "   - Server firewall blocking"
    echo "   - Cloud provider security group"
    echo "   - Service not listening on $PORT"
fi
echo ""

echo "7. Testing HTTP/2 support:"
if curl -k -v --http2 --connect-timeout 10 --max-time 30 https://$HOST:$PORT 2>&1 | grep -q "HTTP/2"; then
    echo "✅ HTTP/2 supported"
else
    echo "⚠️  HTTP/2 not detected (may fall back to HTTP/1.1)"
fi
echo ""

echo "8. Testing API endpoint (if applicable):"
if curl -k --connect-timeout 10 --max-time 30 https://$HOST:$PORT/api/health 2>&1 | grep -q -E "(ok|healthy|success)"; then
    echo "✅ API health check passed"
else
    echo "⚠️  API health check failed or endpoint not available"
    echo "   Trying root endpoint:"
    curl -k --connect-timeout 10 --max-time 30 https://$HOST:$PORT/ 2>&1 | head -5
fi
echo ""

echo "=== Summary ==="
echo "Host: $HOST"
echo "IP: $IP"
echo "Port: $PORT"
echo ""
echo "If connection fails, check:"
echo "1. Server is running and listening on port $PORT"
echo "2. Firewall allows incoming connections on port $PORT"
echo "3. Cloud provider security groups allow port $PORT"
echo "4. SSL certificate is valid (use -k flag to ignore for testing)"
echo "5. Service is configured to listen on 0.0.0.0:$PORT (not just localhost)"

