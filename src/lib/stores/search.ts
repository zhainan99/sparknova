import { writable } from 'svelte/store';

interface ResultItem {
  name: string;
  path: string;
}

interface SearchState {
  query: string;
  results: ResultItem[];
  selectedIndex: number;
}

export const searchStore = writable<SearchState>({
  query: '',
  results: [],
  selectedIndex: 0
});
