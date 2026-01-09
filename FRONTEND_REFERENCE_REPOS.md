# Frontend Reference Repositories

A curated list of repositories to reference for frontend development, especially for wallet UIs and Tauri apps.

## 🪙 Zcash & Cryptocurrency Wallet Frontends

### Zashi Wallet
- **Repository**: Search GitHub for "zashi" or check Zcash Foundation projects
- **Tech Stack**: Likely React/TypeScript
- **Features**: Modern Zcash wallet UI, shielded transactions

### Ywallet (Ycash)
- **GitHub**: `ycashfoundation/ywallet` (if available)
- **Tech Stack**: React Native / React
- **Features**: Mobile-first wallet design

### ZecWallet
- **GitHub**: Search for "zecwallet" repositories
- **Tech Stack**: Electron + React (similar to Tauri)
- **Features**: Desktop wallet with good UI patterns

### Other Crypto Wallet UIs
- **MetaMask**: `MetaMask/metamask-extension` - Excellent React patterns
- **Rainbow Wallet**: `rainbow-me/rainbow` - Beautiful modern UI
- **Trust Wallet**: `trustwallet/wallet-core` - Good component patterns

## 🖥️ Tauri + React Examples

### Official Tauri Examples
- **Tauri Examples**: `tauri-apps/tauri` (examples folder)
- **Tauri React Template**: `tauri-apps/create-tauri-app` - Official starter
- **Tauri Vite React**: `tauri-apps/tauri-vite-react-template`

### Community Tauri Projects
- **Tauri React Boilerplate**: Search GitHub for "tauri-react-boilerplate"
- **Tauri + React + TypeScript**: Many community templates available

## 💼 Wallet-Specific UI Patterns

### Dashboard Components
- **Shadcn UI**: `shadcn/ui` - Beautiful React components
- **Radix UI**: `radix-ui/primitives` - Accessible component library
- **Chakra UI**: `chakra-ui/chakra-ui` - Great for dashboards

### Transaction Lists
- **React Table**: `tanstack/react-table` - Powerful table component
- **Virtual Lists**: `react-window` - For large transaction lists

### Form Components
- **React Hook Form**: `react-hook-form/react-hook-form` - Form handling
- **Zod**: `colinhacks/zod` - TypeScript-first schema validation

## 🎨 Modern UI Libraries

### Component Libraries
- **Mantine**: `mantine/mantine` - Full-featured React components
- **Ant Design**: `ant-design/ant-design` - Enterprise-grade UI
- **Material-UI**: `mui/material-ui` - Google Material Design
- **Tailwind UI**: `tailwindlabs/tailwindui` - Beautiful components

### Design Systems
- **Carbon Design System**: `carbon-design-system/carbon` - IBM's design system
- **Atlassian Design System**: `atlassian/atlaskit` - Professional components

## 🔐 Security & Privacy UI Patterns

### Password/Seed Phrase Components
- **React Password Strength**: `mmoore64/react-password-strength`
- **Mnemonic Input**: Look for "bip39" or "mnemonic" components

### Privacy-Focused UI
- **Tor Browser UI**: Reference for privacy-first design
- **Signal App**: `signalapp/Signal-Desktop` - Privacy-focused patterns

## 📱 Mobile Wallet References

### React Native Wallets
- **Trust Wallet Mobile**: Good mobile wallet patterns
- **Coinbase Wallet**: Mobile-first design patterns

## 🛠️ Development Tools

### State Management
- **Zustand**: `pmndrs/zustand` - Lightweight state management
- **Jotai**: `pmndrs/jotai` - Atomic state management
- **Redux Toolkit**: `reduxjs/redux-toolkit` - If you need Redux

### Routing
- **React Router**: `remix-run/react-router` - You're already using this
- **TanStack Router**: `tanstack/router` - Type-safe routing

## 🎯 Specific Features to Reference

### Balance Display
- Look at MetaMask's balance component
- Check Rainbow Wallet's balance formatting

### Transaction History
- Reference MetaMask's transaction list
- Check ZecWallet's transaction UI

### Address Generation/Display
- Look at wallet QR code components
- Reference address copy/paste patterns

### Send/Receive Forms
- MetaMask's send flow
- Trust Wallet's receive screen

## 🔍 How to Find More

### GitHub Search Queries
```
"zcash wallet" language:typescript
"tauri react" stars:>100
"crypto wallet ui" language:javascript
"react dashboard" wallet
```

### Topics to Browse
- `zcash`
- `tauri`
- `react-components`
- `wallet-ui`
- `cryptocurrency-wallet`

## 📚 Learning Resources

### React Best Practices
- **React Patterns**: `reactpatterns.com`
- **React TypeScript Cheatsheet**: `typescript-cheatsheets/react`

### Tauri Documentation
- **Tauri Docs**: `tauri.app` - Official documentation
- **Tauri Examples**: Check the examples folder in main repo

## 🎨 Design Inspiration

### Dribbble/Behance
- Search "crypto wallet UI"
- Search "dashboard design"
- Search "financial app UI"

### Design Systems
- **Stripe Dashboard**: Reference for financial UIs
- **Coinbase Design**: Professional crypto UI patterns

---

## Quick Start: Clone and Study

```bash
# Example: Clone a Tauri React template
git clone https://github.com/tauri-apps/tauri-vite-react-template
cd tauri-vite-react-template
npm install
npm run tauri dev

# Study the structure:
# - src/App.tsx - Main app structure
# - src/components/ - Component patterns
# - src-tauri/src/ - Backend commands
```

---

**Note**: Always check licenses before using code. Most of these are for reference and learning purposes.

