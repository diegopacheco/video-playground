import { decodeTokenPayload } from "../scenarios";
import type { LicenseExchange, Session } from "../types";

interface Props {
  session: Session | null;
  license: LicenseExchange;
}

export function LicensePanel({ session, license }: Props) {
  return (
    <section className="card">
      <h2>Clear Key exchange</h2>
      <div className="grid3">
        <div>
          <h3>Session token claims</h3>
          <pre>{session ? JSON.stringify(decodeTokenPayload(session.token), null, 2) : "-"}</pre>
        </div>
        <div>
          <h3>License request (CDM)</h3>
          <pre>{license.request || "-"}</pre>
        </div>
        <div>
          <h3>License response (JWK set)</h3>
          <pre>{license.response || "-"}</pre>
        </div>
      </div>
    </section>
  );
}
