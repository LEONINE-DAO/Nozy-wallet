import { PageHeader } from "../components/PageHeader";
import { VoteWizardCard } from "../components/VoteWizardCard";
import type { TabId } from "../components/Header";

interface VotePageProps {
  onNavigate?: (tab: TabId) => void;
}

export function VotePage({ onNavigate }: VotePageProps) {
  return (
    <div className="flex flex-col gap-8 animate-fade-in w-full pb-4">
      <PageHeader
        title="NU7 Vote"
        description="Coinholder ballot (Valar Shielded Vote). Eligible weight = spendable Ironwood notes at the Aug 24 2026 snapshot. Seed stays in this wallet; voting hotkey is separate."
      />
      <VoteWizardCard onNavigateIronwood={() => onNavigate?.("ironwood")} />
    </div>
  );
}
