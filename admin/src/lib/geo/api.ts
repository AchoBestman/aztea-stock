export interface CountryOption {
  code: string;
  name: string;
}

interface RestCountry {
  cca2: string;
  name: { common: string };
}

interface CountriesNowCountry {
  name: string;
  Iso2: string;
}


interface CountriesNowTimezoneItem {
  zoneName: string;
  gmtOffsetName: string;
}

interface CountriesNowTimezoneResponse {
  error: boolean;
  msg: string;
  data: { timezones: CountriesNowTimezoneItem[] };
}

interface CountriesNowEntry {
  country: string;
  cities: string[];
}

// ── Caches ────────────────────────────────────────────────────────────────────

let countriesCache: CountryOption[] | null = null;
/** ISO2 → CountriesNow country name, loaded once */
let isoNameCache: Map<string, string> | null = null;
/** Full GET /countries payload — loaded once, used for city lookups */
let countriesNowDataCache: CountriesNowEntry[] | null = null;

// ── Public API ────────────────────────────────────────────────────────────────

export async function fetchCountries(): Promise<CountryOption[]> {
  if (countriesCache) return countriesCache;
  const res = await fetch("https://restcountries.com/v3.1/all?fields=cca2,name");
  if (!res.ok) throw new Error("Impossible de charger les pays");
  const data = (await res.json()) as RestCountry[];
  countriesCache = data
    .map((c) => ({ code: c.cca2, name: c.name.common }))
    .sort((a, b) => a.name.localeCompare(b.name, "fr"));
  return countriesCache;
}

/** restcountries.com common name for a given ISO-2 code */
export async function countryNameForApi(isoCode: string): Promise<string> {
  const countries = await fetchCountries();
  return countries.find((c) => c.code === isoCode)?.name || isoCode;
}

/**
 * Resolve ISO-2 → the exact name CountriesNow expects.
 * The ISO map is fetched once and cached for the session.
 */
export async function resolveCountryName(isoCode: string): Promise<string> {
  if (!isoNameCache) {
    try {
      const res = await fetch("https://countriesnow.space/api/v0.1/countries/iso");
      const json = (await res.json()) as { error: boolean; data: CountriesNowCountry[] };
      if (!json.error && Array.isArray(json.data)) {
        isoNameCache = new Map(
          json.data.map((c) => [c.Iso2.toUpperCase(), c.name])
        );
      } else {
        isoNameCache = new Map();
      }
    } catch {
      isoNameCache = new Map();
    }
  }

  const fromIso = isoNameCache.get(isoCode.toUpperCase());
  // Verify the ISO-derived name actually exists in the /countries dataset;
  // if it doesn't, fall back to the common name and let fuzzy matching handle it.
  if (fromIso) {
    const data = await getCountriesNowData();
    const nFromIso = normName(fromIso);
    const match = data.find(
      (e) => normName(e.country) === nFromIso ||
             normName(e.country).includes(nFromIso) ||
             nFromIso.includes(normName(e.country))
    );
    if (match) return match.country; // use the exact spelling from the dataset
  }

  // Last resort: restcountries.com common name (fuzzy matching in fetchCities will still help)
  return countryNameForApi(isoCode);
}

/** Normalise a name for fuzzy comparison: lowercase, remove diacritics, collapse punctuation. */
function normName(s: string): string {
  return s
    .toLowerCase()
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/['\-.,]/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

/** Load and cache the full GET /countries list (all countries + cities). */
async function getCountriesNowData(): Promise<CountriesNowEntry[]> {
  if (countriesNowDataCache) return countriesNowDataCache;
  try {
    const res = await fetch("https://countriesnow.space/api/v0.1/countries");
    const json = (await res.json()) as { error: boolean; data: CountriesNowEntry[] };
    if (!json.error && Array.isArray(json.data)) {
      countriesNowDataCache = json.data;
      return countriesNowDataCache;
    }
  } catch { /* ignore */ }
  countriesNowDataCache = [];
  return countriesNowDataCache;
}

/**
 * Fetch cities for a country name.
 * Uses the cached GET /countries list — no extra API calls after first load.
 * Matching strategy: exact → normalised exact → one-side-contains.
 */
export async function fetchCitiesByCountryName(
  countryName: string
): Promise<string[]> {
  const data = await getCountriesNowData();
  const needle = normName(countryName);

  // 1. Exact lowercase match
  let entry = data.find((e) => e.country.toLowerCase() === countryName.toLowerCase());

  // 2. Normalised exact (strips diacritics / punctuation)
  if (!entry) entry = data.find((e) => normName(e.country) === needle);

  // 3. One-side contains (handles "Bosnia And Herzegovina" ↔ "Bosnia and Herzegovina")
  if (!entry) {
    entry = data.find(
      (e) =>
        normName(e.country).includes(needle) ||
        needle.includes(normName(e.country))
    );
  }

  if (!entry || entry.cities.length === 0) return [];
  return entry.cities.sort((a, b) => a.localeCompare(b, "fr"));
}

/**
 * Fetch timezones for a country name.
 * Returns an empty array (not throw) on failure.
 */
export async function fetchTimezonesByCountryName(
  countryName: string
): Promise<string[]> {
  try {
    const res = await fetch(
      "https://countriesnow.space/api/v0.1/countries/timezone",
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ country: countryName }),
      }
    );
    const json = (await res.json()) as CountriesNowTimezoneResponse;
    if (json.error || !json.data?.timezones?.length) return [];
    return json.data.timezones
      .map((t) => t.zoneName)
      .sort((a, b) => a.localeCompare(b));
  } catch {
    return [];
  }
}
