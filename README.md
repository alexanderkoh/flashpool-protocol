# FlashPool: Decentralized Liquidity Campaign Platform

FlashPool is a platform designed to enable users to create and participate in liquidity incentive campaigns for various token pairs, with rewards distributed in FLASH tokens. It aims to provide a user-friendly interface for launching campaigns, exploring active campaigns, and managing wallet connections for participation.

## Tech Stack

*   **Framework:** Next.js (App Router)
*   **Language:** TypeScript
*   **Styling:** Tailwind CSS
*   **UI Components:** Shadcn/UI
*   **Charting:** Recharts
*   **Wallet Integration:** Freighter (currently mocked, with `useWallet` hook for abstraction)
*   **State Management:** React Context API (for wallet state), `useState`, `useEffect`, `useCallback` for component-level state.

## Getting Started

1.  **Clone the repository:**
    ```bash
    git clone <repository-url>
    cd FlashPool
    ```

2.  **Install dependencies:**
    ```bash
    npm install
    # or
    yarn install
    ```

3.  **Run the development server:**
    ```bash
    npm run dev
    # or
    yarn dev
    ```
    Open [http://localhost:3000](http://localhost:3000) with your browser to see the result.

## Project Structure

```
FlashPool/
├── app/                  # Next.js App Router: Pages and layouts
│   ├── (main)/           # Main application routes
│   │   ├── campaign/
│   │   │   ├── [id]/page.tsx     # Campaign detail page
│   │   │   └── new/page.tsx      # Launch new campaign page
│   │   ├── explore/page.tsx    # Explore campaigns page
│   │   ├── rewards/page.tsx    # User rewards page
│   │   └── ...               # Other main pages (home, etc.)
│   ├── admin/              # Admin specific routes/pages
│   ├── layout.tsx          # Root layout
│   └── page.tsx            # Home page
├── components/
│   ├── campaign/           # Campaign-specific UI components (form, card, grid, stats)
│   ├── layout/             # Layout components (navbar, footer)
│   ├── ui/                 # Reusable UI components (button, input, card, etc. from Shadcn/UI)
│   └── ...                 # Other shared components
├── hooks/                # Custom React hooks (e.g., useWallet.ts, useToast.ts)
├── lib/
│   ├── constants.ts        # Application constants (e.g., menu items)
│   ├── freighter-provider.tsx # Wallet context and mock logic
│   ├── utils.ts            # Utility functions (formatting, cn, etc.)
│   └── ...                 # Other library code
├── public/               # Static assets (images, fonts)
├── styles/               # Global styles (globals.css)
├── .env.local.example    # Environment variable examples
├── next.config.js        # Next.js configuration
├── tailwind.config.js    # Tailwind CSS configuration
├── tsconfig.json         # TypeScript configuration
└── README.md             # This file
```

## Key Features (Implemented & In Progress)

*   **Campaign Creation:** Multi-step form to launch new liquidity incentive campaigns.
    *   Pool selection.
    *   Contribution token selection and amount input.
    *   Automatic quoting of contribution to FLASH rewards.
    *   Setting campaign duration, target TVL.
    *   Configurable boost periods (start, duration, multiplier).
    *   Estimated APY calculation based on FLASH rewards.
    *   Summary before launch.
*   **Campaign Exploration:**
    *   Grid and card view for active/available campaigns.
    *   Display of key campaign metrics (name, pair, rewards, TVL, time left, APY).
    *   Visual indication for boosted campaigns.
*   **Wallet Integration (Mocked Freighter):**
    *   Connect/disconnect wallet functionality.
    *   Display of connected wallet public key.
    *   Shared wallet state via `FreighterProvider` and `useWallet` hook.
*   **UI & UX:**
    *   Responsive design for desktop and mobile.
    *   Consistent UI using Shadcn/UI components.
    *   Client-side navigation with Next.js App Router.
    *   Loading states and skeleton screens for better perceived performance.

## TODO / Future Enhancements

### Core Functionality & Backend
*   **[ ] Integrate Stellar Wallet Kit:** Replace mock Freighter logic with a robust Stellar wallet solution (e.g., WalletConnect, Albedo, or full Freighter SDK integration).
*   **[ ] Create Contract Bindings:** Develop and integrate bindings for Soroban/Stellar smart contracts that will manage campaign logic, deposits, and reward distribution.
*   **[ ] Add Contract Folder:** Create a dedicated `/contracts` folder for Soroban/Stellar smart contract source code and related artifacts (e.g., WASM, ABI).
*   **[ ] Implement Real Backend & Database:** 
    *   Replace all mock data (`availablePools`, campaign data in `CampaignGrid`, chart data in `CampaignStats`, etc.) with actual data fetched from a backend.
    *   User authentication and profiles.
    *   Persistence of campaign data, user contributions, and reward states.
*   **[ ] Real-time Data & Progress:**
    *   Implement logic to fetch and display real-time campaign progress (e.g., actual hours passed, current TVL contributions).
    *   Accurate, dynamic APY calculations based on real-time on-chain or backend data.
*   **[ ] Transaction Signing and Submission:** Implement actual transaction flows for launching campaigns, contributing liquidity, and claiming rewards.
*   **[ ] Real Exchange Rate Fetching:** Replace mock `getFlashEquivalent` with calls to a reliable oracle or DEX API for accurate token to FLASH quoting.

### Frontend & UI/UX
*   **[ ] Define Strong TypeScript Interfaces:** Replace `any` types (e.g., for `campaignData` in `CampaignForm`) with well-defined TypeScript interfaces for better type safety and maintainability.
*   **[ ] Comprehensive Form Validation & Error Handling:** Implement robust client-side and server-side validation for all forms (campaign creation, deposits, etc.) with clear user feedback.
*   **[ ] Advanced Filtering & Sorting:** Enhance the "Explore Campaigns" page with more sophisticated filtering (by token, APY range, status) and sorting options.
*   **[ ] User Profile/Dashboard:** Create a dedicated area for users to view their created campaigns, participation history, and earned/claimable rewards.
*   **[ ] Notifications System:** Implement a system for user notifications (e.g., campaign starting/ending, rewards claimable) using toasts or a dedicated notification center.
*   **[ ] Finalize UI/UX:** Conduct a thorough review of all pages and components to ensure consistency, responsiveness across all device sizes, and optimal user experience.
*   **[ ] Accessibility (A11y) Audit & Improvements:** Ensure the application is accessible by following WCAG guidelines and performing accessibility testing.
*   **[ ] Mobile Navbar Link Visibility:** Confirm and ensure mobile navigation links are always visible with proper contrast against the menu background.

### Development & Operations
*   **[ ] Unit & Integration Tests:** Implement a testing strategy with frameworks like Jest and React Testing Library to ensure code quality and prevent regressions.
*   **[ ] Deployment Setup & CI/CD:** Configure a deployment pipeline (e.g., Vercel, Netlify, AWS Amplify) and set up Continuous Integration/Continuous Deployment.
*   **[ ] Detailed API Documentation:** If custom backend APIs are developed, create comprehensive documentation (e.g., using Swagger/OpenAPI).

## Contributing

Details on how to contribute to the project will be added here. (e.g., Fork the repo, create a feature branch, submit a PR).

## License

This project is licensed under the MIT License. (Or specify your chosen license). 