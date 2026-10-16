export interface Title {
  id: number;
  name: string;
  genres: string[];
  tags: string[];
  synopsis: string;
  year: number;
  is_new: boolean;
}

export interface TitleView {
  title: Title;
  watches: number;
}

export interface Item {
  title: Title;
  score: number;
}

export type RowKind = "top" | "because" | "similar" | "popular";

export interface Row {
  kind: RowKind;
  label: string;
  anchor: number | null;
  items: Item[];
}

export type Algorithm = "hybrid" | "cf" | "content" | "popular";

export interface Recommendations {
  algorithm: Algorithm;
  cold_start: boolean;
  rows: Row[];
}

export interface TitleDetail {
  title: Title;
  watches: number;
  similar_content: Item[];
  also_watched: Item[];
}

export interface User {
  id: number;
  name: string;
  cluster: string | null;
  picked_genres: string[];
}

export interface UserView {
  user: User;
  watch_count: number;
}

export interface UserDetail {
  user: User;
  watched: Title[];
}

export interface Cluster {
  name: string;
  genres: string[];
}

export interface Genres {
  genres: string[];
  clusters: Cluster[];
}

export interface EvalRow {
  scenario: string;
  algorithm: Algorithm;
  precision_at_10: number;
  recall_at_10: number;
  users: number;
}

export interface NewTitle {
  name: string;
  genres: string[];
  tags: string[];
  synopsis: string;
  year: number;
}
