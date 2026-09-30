import { PageHeader } from "../components/PageHeader";
import { IronwoodReadinessCard } from "../components/IronwoodReadinessCard";

export function IronwoodPage() {
  return (
    <div className="flex flex-col gap-8 animate-fade-in w-full pb-4">
      <PageHeader
        title="Ironwood"
        description="Migrate residual Orchard, or sync if you already received Ironwood. Prefer local Zebrad."
      />
      <IronwoodReadinessCard />
    </div>
  );
}
