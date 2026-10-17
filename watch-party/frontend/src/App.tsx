import { useState } from "react";
import { JoinForm } from "./components/JoinForm";
import { Party } from "./components/Party";

interface Session {
  room: string;
  name: string;
}

export function App() {
  const [session, setSession] = useState<Session | null>(null);
  return (
    <main className="app">
      <h1>Watch Party</h1>
      {session ? (
        <Party room={session.room} name={session.name} onLeave={() => setSession(null)} />
      ) : (
        <JoinForm onJoin={(room, name) => setSession({ room, name })} />
      )}
    </main>
  );
}
