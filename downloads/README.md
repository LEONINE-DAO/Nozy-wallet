# NozyWallet Downloads Page


### 1. Get the File

```bash
git clone https://github.com/LEONINE-DAO/Nozy-wallet.git
cd Nozy-wallet/downloads
```

### 2. Edit

Open `index.html` in your favorite editor and make changes.

### 3. Test Locally

```bash
python3 -m http.server 8000
# Then open http://localhost:8000/index.html in your browser
```

### 4. Submit Changes

```bash
git checkout -b frontend/updates

git add index.html
git commit -m "Update downloads page design"

git push origin frontend/updates
```

## Structure

The page is a single HTML file with:
- **HTML** - Structure and content
- **CSS** - All styles in `<style>` tag
- **JavaScript** - Dynamic content loading

## Key Features

- Fetches latest release from GitHub API
- Displays download links for all platforms
- Shows SHA256 hashes for verification
- Responsive design (mobile-friendly)
- Auto-updates when new releases are published

## Deployment

Automatically deployed to GitHub Pages when:
- Changes are pushed to `main` branch
- New releases are published

Live at: `https://LEONINE-DAO.github.io/Nozy-wallet/`

## Need Help?

See `FRONTEND_DEVELOPER_GUIDE.md` in the root directory for detailed instructions.

