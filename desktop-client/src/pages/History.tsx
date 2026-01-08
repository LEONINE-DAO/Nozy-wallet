import { ArrowRightUp, ArrowLeftDown, Calendar } from "@solar-icons/react";

// Mock data for history
const HISTORY_DATA = [
  {
    id: "1",
    type: "received",
    amount: 12.5,
    date: "2024-03-15",
    address: "88...9x2a",
    status: "confirmed",
  },
  {
    id: "2",
    type: "sent",
    amount: 4.2,
    date: "2024-03-14",
    address: "44...k8p1",
    status: "confirmed",
  },
  {
    id: "3",
    type: "received",
    amount: 100.0,
    date: "2024-03-10",
    address: "99...m2z5",
    status: "confirmed",
  },
  {
    id: "4",
    type: "sent",
    amount: 1.5,
    date: "2024-03-05",
    address: "33...j4r9",
    status: "pending",
  },
];

export function HistoryPage() {
  return (
    <div className="space-y-6 animate-fade-in max-w-4xl mx-auto">
      <div className="flex items-center justify-between">
        <h2 className="text-3xl font-bold text-gray-900">
          Transaction History
        </h2>
      </div>

      <div className="bg-white/60 backdrop-blur-md rounded-2xl border border-white/50 shadow-sm overflow-hidden">
        {HISTORY_DATA.length > 0 ? (
          <div className="divide-y divide-gray-100/50">
            {HISTORY_DATA.map((tx) => (
              <div
                key={tx.id}
                className="p-4 hover:bg-white/40 transition-colors flex items-center justify-between group"
              >
                <div className="flex items-center gap-4">
                  <div
                    className={`w-10 h-10 rounded-full flex items-center justify-center ${
                      tx.type === "received"
                        ? "bg-green-100 text-green-600"
                        : "bg-red-100 text-red-600"
                    }`}
                  >
                    {tx.type === "received" ? (
                      <ArrowLeftDown size={20} />
                    ) : (
                      <ArrowRightUp size={20} />
                    )}
                  </div>
                  <div>
                    <p className="font-semibold text-gray-900">
                      {tx.type === "received" ? "Received" : "Sent"}
                    </p>
                    <div className="flex items-center gap-2 text-xs text-gray-500">
                      <Calendar size={12} />
                      <span>{tx.date}</span>
                      <span className="w-1 h-1 rounded-full bg-gray-300" />
                      <span className="font-mono">{tx.address}</span>
                    </div>
                  </div>
                </div>

                <div className="text-right">
                  <p
                    className={`font-bold uppercase ${
                      tx.type === "received"
                        ? "text-green-600"
                        : "text-gray-900"
                    }`}
                  >
                    {tx.type === "received" ? "+" : "-"}
                    {tx.amount.toFixed(4)} Zec
                  </p>
                  <span
                    className={`text-xs px-2 py-0.5 rounded-full ${
                      tx.status === "confirmed"
                        ? "bg-green-100 text-green-700"
                        : "bg-yellow-100 text-yellow-700"
                    }`}
                  >
                    {tx.status}
                  </span>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="p-12 text-center text-gray-500">
            <p>No transactions found</p>
          </div>
        )}
      </div>
    </div>
  );
}
