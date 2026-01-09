#!/bin/bash
# Password Reset Helper Script for WSL/Linux
# This script helps you reset your wallet password by restoring from mnemonic

echo "🔐 NozyWallet Password Reset Helper"
echo "===================================="
echo ""

echo "⚠️  IMPORTANT: You need your mnemonic phrase to reset the password!"
echo ""

read -p "Do you have your 12/24 word mnemonic phrase? (y/n): " has_mnemonic

if [ "$has_mnemonic" != "y" ] && [ "$has_mnemonic" != "Y" ]; then
    echo ""
    echo "❌ Without your mnemonic phrase, the wallet cannot be recovered."
    echo "   The wallet is encrypted and requires either:"
    echo "   1. The correct password, OR"
    echo "   2. The mnemonic phrase to restore"
    echo ""
    echo "💡 If you lost both, the wallet and funds are permanently lost."
    exit 1
fi

echo ""
echo "✅ Great! Let's restore your wallet with a new password."
echo ""
echo "📝 Steps:"
echo "   1. You'll be prompted for your mnemonic phrase"
echo "   2. You'll be prompted for a NEW password (or leave empty for no password)"
echo "   3. Your wallet will be restored with the new password"
echo ""

read -p "Ready to proceed? (y/n): " confirm

if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
    echo "Cancelled."
    exit 0
fi

echo ""
echo "🔄 Running restore command..."
echo ""

# Run the restore command
./target/release/nozy restore

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Password reset successful!"
    echo "   Your wallet now uses the new password you set."
else
    echo ""
    echo "❌ Restore failed. Please check:"
    echo "   - Mnemonic phrase is correct (12 or 24 words)"
    echo "   - No typos in the mnemonic"
    echo "   - Wallet file is not corrupted"
fi
