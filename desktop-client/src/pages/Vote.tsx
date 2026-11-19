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
        title="Coinholder vote"
        description="Open coinholder poll. Ironwood weight at the live snapshot. Seed stays here; cast after delegation."
      />
      <VoteWizardCard onNavigateIronwood={() => onNavigate?.("ironwood")} />
    </div>
  );
}
