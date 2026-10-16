import type { AcquireResult, Account, Lease, Policy, Snapshot } from "../types";

async function json<T>(res: Response): Promise<T> {
  if (!res.ok) {
    throw new Error(`${res.status} ${await res.text()}`);
  }
  return (await res.json()) as T;
}

function devicePath(account: string, device: string, action: string): string {
  return `/api/accounts/${encodeURIComponent(account)}/devices/${encodeURIComponent(device)}/${action}`;
}

export async function fetchAccounts(): Promise<{ accounts: Account[]; ttl_ms: number }> {
  return json(await fetch("/api/accounts"));
}

export async function fetchPolicy(): Promise<Policy> {
  const body = await json<{ policy: Policy }>(await fetch("/api/policy"));
  return body.policy;
}

export async function updatePolicy(policy: Policy): Promise<Policy> {
  const res = await fetch("/api/policy", {
    method: "PUT",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ policy }),
  });
  const body = await json<{ policy: Policy }>(res);
  return body.policy;
}

export async function fetchSnapshot(account: string): Promise<Snapshot> {
  return json(await fetch(`/api/accounts/${encodeURIComponent(account)}/leases`));
}

export async function acquire(account: string, device: string): Promise<AcquireResult> {
  const res = await fetch(devicePath(account, device, "acquire"), { method: "POST" });
  if (res.status === 409) {
    return (await res.json()) as AcquireResult;
  }
  return json(res);
}

export async function heartbeat(account: string, device: string): Promise<Lease | null> {
  const res = await fetch(devicePath(account, device, "heartbeat"), { method: "POST" });
  if (res.status === 410) {
    return null;
  }
  const body = await json<{ lease: Lease }>(res);
  return body.lease;
}

export async function release(account: string, device: string): Promise<boolean> {
  const body = await json<{ released: boolean }>(
    await fetch(devicePath(account, device, "release"), { method: "POST" }),
  );
  return body.released;
}
