export type Field = "romaji" | "english" | "japanese";

export interface Segment {
  text: string;
  hit: boolean;
}

export interface FieldHighlight {
  field: Field;
  segments: Segment[];
}

export interface Breakdown {
  field: Field;
  similarity: number;
  coverage: number;
  edit_distance: number;
  overlap: number;
  boost: number;
  popularity: number;
  similarity_score: number;
  coverage_score: number;
  text_score: number;
  popularity_score: number;
  total: number;
}

export interface Hit {
  id: number;
  romaji: string;
  english: string;
  japanese: string;
  year: number;
  popularity: number;
  score: number;
  breakdown: Breakdown;
  highlights: FieldHighlight[];
}

export interface SearchResponse {
  query: string;
  normalized: string;
  grams: string[];
  candidates: number;
  took_us: number;
  results: Hit[];
}

export interface IndexStats {
  documents: number;
  grams: number;
  postings: number;
}
