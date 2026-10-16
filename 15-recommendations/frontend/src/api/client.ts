import type {
  Algorithm,
  EvalRow,
  Genres,
  NewTitle,
  Recommendations,
  Title,
  TitleDetail,
  TitleView,
  User,
  UserDetail,
  UserView,
} from "../types";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`/api${path}`, {
    ...init,
    headers: { "content-type": "application/json", ...(init?.headers ?? {}) },
  });
  if (!res.ok) {
    const body = (await res.json().catch(() => ({}))) as { error?: string };
    throw new Error(body.error ?? `${res.status} ${res.statusText}`);
  }
  if (res.status === 204) {
    return undefined as T;
  }
  return (await res.json()) as T;
}

export const api = {
  genres: () => request<Genres>("/genres"),
  eval: () => request<EvalRow[]>("/eval"),
  titles: () => request<TitleView[]>("/titles"),
  title: (id: number) => request<TitleDetail>(`/titles/${id}`),
  createTitle: (body: NewTitle) => request<Title>("/titles", { method: "POST", body: JSON.stringify(body) }),
  users: () => request<UserView[]>("/users"),
  user: (id: number) => request<UserDetail>(`/users/${id}`),
  createUser: (name: string, genres: string[]) =>
    request<User>("/users", { method: "POST", body: JSON.stringify({ name, genres }) }),
  watch: (userId: number, titleId: number) =>
    request<void>(`/users/${userId}/watches`, { method: "POST", body: JSON.stringify({ title_id: titleId }) }),
  recommendations: (userId: number, algo: Algorithm) =>
    request<Recommendations>(`/users/${userId}/recommendations?algo=${algo}`),
};
