# NozyWallet Landing Page

A premium, modern landing page for NozyWallet, built with React, TypeScript, and Tailwind CSS. This project serves as the primary gateway for NozyWallet users, highlighting security, features, and ease of use.

## 🚀 Tech Stack

- **React 19**: Latest version of React for building the UI.
- **TypeScript**: For type-safe development.
- **Vite**: Ultra-fast build tool and development server.
- **Tailwind CSS 4**: Modern styling with utility-first classes.
- **Solar Icons**: Elegant iconography from `@solar-icons/react`.
- **Vercel Analytics**: Built-in performance and visitor tracking.

## ✨ Features

- **Modern UI/UX**: Clean, responsive, and vibrant design.
- **Dynamic FAQ**: Fully interactive FAQ section for user inquiries.
- **Security & Privacy**: Dedicated pages for security protocols and privacy policies.
- **Highly Responsive**: Optimized for desktop, tablet, and mobile devices.

## 🛠️ Development

To get started with the project locally:

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/LEONINE-DAO/NozyWallet-landing.git
    cd NozyWallet-landing
    ```

2.  **Install dependencies**:
    ```bash
    npm install
    ```

3.  **Run in development mode**:
    ```bash
    npm run dev
    ```

4.  **Open the browser**:
    Navigate to `http://localhost:5173`.

## 📦 Deployment (IMPORTANT)

This project is configured to be deployed via **GitHub Pages** using the `docs` folder.

**CRITICAL:** Before pushing your changes to GitHub, you **MUST** run the deployment script to rebuild the production files and update the `docs` directory.

### Deployment Workflow:

1.  Make your changes in the `src` directory.
2.  Run the deploy command:
    ```bash
    npm run deploy
    ```
    *This script will: build the project, remove the old `docs` folder, and move the new `dist` contents into `docs`.*
3.  Stage and commit your changes (including the updated `docs` folder):
    ```bash
    git add .
    git commit -m "Update: [your changes description]"
    ```
4.  Push to the main branch:
    ```bash
    git push origin main
    ```

GitHub Pages will automatically pick up the changes from the `docs` folder and update the live site.

## 🤝 Maintenance

- **Adding Icons**: Use `@solar-icons/react` for consistency.
- **Updating Styles**: Tailwind configuration is located in `vite.config.ts` (using `@tailwindcss/vite`).
- **Pages**: New pages can be added in `src/pages` and routed in `src/App.tsx`.

---

Built with ❤️ for NozyWallet.
