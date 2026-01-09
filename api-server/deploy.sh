set -e

echo "🚀 Deploying NozyWallet API Server..."

if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the api-server directory"
    exit 1
fi

echo "📦 Building release binary..."
cd ..
cargo build --release --bin nozywallet-api

echo "🛑 Stopping existing service..."
sudo systemctl stop nozywallet-api 2>/dev/null || true

echo "📋 Installing binary..."
sudo mkdir -p /opt/nozywallet
sudo cp api-server/target/release/nozywallet-api /opt/nozywallet/
sudo chmod +x /opt/nozywallet/nozywallet-api

echo "🔄 Starting service..."
sudo systemctl start nozywallet-api
sleep 2
sudo systemctl status nozywallet-api --no-pager

echo "✅ Deployment complete!"
echo "📊 Check logs with: sudo journalctl -u nozywallet-api -f"
