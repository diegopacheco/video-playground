import { useCallback, useEffect, useState } from "react";
import { fetchAccounts, fetchPolicy, updatePolicy } from "./api/client";
import { AccountTabs } from "./components/AccountTabs";
import { DeviceGrid } from "./components/DeviceGrid";
import { EventLog } from "./components/EventLog";
import { LeaseTable } from "./components/LeaseTable";
import { PolicyToggle } from "./components/PolicyToggle";
import { useDevices } from "./hooks/useDevices";
import { useNow } from "./hooks/useNow";
import { useSnapshot } from "./hooks/useSnapshot";
import type { Account, LogEntry, Policy } from "./types";

export function App() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [ttl, setTtl] = useState(10_000);
  const [selected, setSelected] = useState("alice");
  const [policy, setPolicy] = useState<Policy | null>(null);
  const [entries, setEntries] = useState<LogEntry[]>([]);
  const [refreshKey, setRefreshKey] = useState(0);

  const log = useCallback((text: string, tone: LogEntry["tone"]) => {
    setEntries((prev) => [{ at: Date.now(), text, tone }, ...prev].slice(0, 40));
    setRefreshKey((k) => k + 1);
  }, []);

  useEffect(() => {
    fetchAccounts()
      .then((body) => {
        setAccounts(body.accounts);
        setTtl(body.ttl_ms);
      })
      .catch((e: Error) => log(`cannot load accounts: ${e.message}`, "bad"));
    fetchPolicy()
      .then(setPolicy)
      .catch((e: Error) => log(`cannot load policy: ${e.message}`, "bad"));
  }, [log]);

  const { devices, start, stop, crash } = useDevices(ttl, log);
  const data = useSnapshot(selected, refreshKey);
  const now = useNow(100);

  const changePolicy = (next: Policy) => {
    updatePolicy(next)
      .then((p) => {
        setPolicy(p);
        log(`policy set to ${p}`, "info");
      })
      .catch((e: Error) => log(`policy update failed: ${e.message}`, "bad"));
  };

  return (
    <main>
      <header className="top">
        <div>
          <h1>Concurrent Stream Limit</h1>
          <p className="muted">
            Leases live in a Redis ZSET per account. Lease ttl {ttl / 1000}s, heartbeat every {Math.floor(ttl / 3000)}s.
          </p>
        </div>
        <PolicyToggle policy={policy} onChange={changePolicy} />
      </header>
      <AccountTabs accounts={accounts} selected={selected} onSelect={setSelected} />
      <DeviceGrid
        account={selected}
        devices={devices}
        onStart={(d) => void start(selected, d)}
        onStop={(d) => void stop(selected, d)}
        onCrash={(d) => crash(selected, d)}
      />
      <div className="columns">
        <LeaseTable data={data} devices={devices} now={now} />
        <EventLog entries={entries} />
      </div>
    </main>
  );
}
