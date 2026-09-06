import { PageHeader } from "../components/PageHeader";
import { CrosslinkGuardianCard } from "../components/CrosslinkGuardianCard";

export function CrosslinkPage() {
  return (
    <div className="flex flex-col gap-8 animate-fade-in w-full pb-4">
      <PageHeader
        title="Crosslink"
        description="Stake and manage Crosslink bonds."
      />
      <CrosslinkGuardianCard />
    </div>
  );
}
