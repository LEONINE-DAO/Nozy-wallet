import { PageHeader } from "../components/PageHeader";
import { IronwoodReadinessCard } from "../components/IronwoodReadinessCard";

export function IronwoodPage() {
  return (
    <div className="flex flex-col gap-8 animate-fade-in w-full pb-4">
      <PageHeader
        title="Ironwood"
        description="NU6.3 readiness and Orchard → Ironwood migration (Plan → Migrate → Broadcast). Prefer local Zebrad; testnet: Ironwood testnet profile + WSL node."
      />
      <IronwoodReadinessCard />
    </div>
  );
}
