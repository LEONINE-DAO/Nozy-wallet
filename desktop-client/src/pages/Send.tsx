import { SendForm } from "../components/SendForm";

export function SendPage() {
  return (
    <div className="max-w-xl mx-auto animate-fade-in pt-10 pb-20">
      <div className="bg-white/80 backdrop-blur-xl rounded-3xl p-8 shadow-xl shadow-primary/5 border border-white/50 relative overflow-hidden">
        <div className="absolute top-0 left-0 w-full h-1 bg-linear-to-r from-transparent via-primary to-transparent" />
        <h2 className="text-2xl font-bold text-gray-900 mb-8 text-center flex items-center justify-center gap-2">
          Send Zec
        </h2>
        <SendForm />
      </div>
    </div>
  );
}
