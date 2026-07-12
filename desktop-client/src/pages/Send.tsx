import { SendForm } from "../components/SendForm";
import { Card } from "../components/Card";
import { PageHeader } from "../components/PageHeader";

export function SendPage() {
  return (
    <div className="max-w-lg mx-auto animate-fade-in pb-8">
      <PageHeader
        title="Send ZEC"
        description="Transfer shielded funds to another address"
        className="mb-6"
      />
      <Card variant="elevated" padding="lg" className="relative overflow-hidden">
        <div className="absolute top-0 left-0 w-full h-0.5 bg-linear-to-r from-transparent via-primary to-transparent" />
        <SendForm />
      </Card>
    </div>
  );
}
