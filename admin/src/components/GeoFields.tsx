import { useEffect, useState } from "react";
import {
  fetchCountries,
  fetchCitiesByCountryName,
  fetchTimezonesByCountryName,
  resolveCountryName,
  type CountryOption,
} from "../lib/geo/api";

export interface GeoValues {
  country: string;       // ISO2 code (e.g. "CG")
  country_name: string;  // full name (e.g. "Congo")
  city: string;
  timezone: string;
}

interface Props {
  value: GeoValues;
  onChange: (v: GeoValues) => void;
  disabled?: boolean;
}

export default function GeoFields({ value, onChange, disabled }: Props) {
  const [countries, setCountries] = useState<CountryOption[]>([]);
  const [cities, setCities] = useState<string[]>([]);
  const [timezones, setTimezones] = useState<string[]>([]);
  const [loadingCountries, setLoadingCountries] = useState(true);
  const [loadingCities, setLoadingCities] = useState(false);
  const [loadingTz, setLoadingTz] = useState(false);
  const [cityFreeText, setCityFreeText] = useState(false);
  const [tzFreeText, setTzFreeText] = useState(false);
  const [cityApiEmpty, setCityApiEmpty] = useState(false);
  const [tzApiEmpty, setTzApiEmpty] = useState(false);

  useEffect(() => {
    fetchCountries()
      .then(setCountries)
      .finally(() => setLoadingCountries(false));
  }, []);

  useEffect(() => {
    if (!value.country) {
      setCities([]);
      setTimezones([]);
      setCityFreeText(false);
      setTzFreeText(false);
      setCityApiEmpty(false);
      setTzApiEmpty(false);
      return;
    }
    let cancelled = false;
    setLoadingCities(true);
    setLoadingTz(true);
    setCityFreeText(false);
    setTzFreeText(false);
    setCityApiEmpty(false);
    setTzApiEmpty(false);

    resolveCountryName(value.country)
      .then(async (countryName) => {
        const [cityList, tzList] = await Promise.all([
          fetchCitiesByCountryName(countryName),
          fetchTimezonesByCountryName(countryName),
        ]);
        if (cancelled) return;

        setCities(cityList);
        setTimezones(tzList);

        // Auto free-text: API returned nothing
        if (cityList.length === 0) {
          setCityApiEmpty(true);
          setCityFreeText(true);
        } else if (value.city && !cityList.includes(value.city)) {
          setCityFreeText(true);
        }

        if (tzList.length === 0) {
          setTzApiEmpty(true);
          setTzFreeText(true);
        } else if (value.timezone && !tzList.includes(value.timezone)) {
          setTzFreeText(true);
        }
      })
      .catch(() => {
        if (cancelled) return;
        setCities([]);
        setTimezones([]);
        setCityApiEmpty(true);
        setTzApiEmpty(true);
        setCityFreeText(true);
        setTzFreeText(true);
      })
      .finally(() => {
        if (!cancelled) {
          setLoadingCities(false);
          setLoadingTz(false);
        }
      });

    return () => { cancelled = true; };
  }, [value.country]);

  return (
    <div className="grid gap-3 sm:grid-cols-3">
      {/* PAYS */}
      <label className="block text-sm">
        <span className="font-medium text-muted-foreground">Pays *</span>
        <select
          required
          disabled={disabled || loadingCountries}
          className="form-select mt-1"
          value={value.country}
          onChange={(e) => {
            const name = countries.find((c) => c.code === e.target.value)?.name ?? e.target.value;
            onChange({ country: e.target.value, country_name: name, city: "", timezone: "" });
          }}
        >
          <option value="">{loadingCountries ? "Chargement…" : "— Choisir —"}</option>
          {countries.map((c) => (
            <option key={c.code} value={c.code}>{c.name}</option>
          ))}
        </select>
      </label>

      {/* VILLE */}
      <div className="block text-sm">
        <span className="font-medium text-muted-foreground flex items-center justify-between mb-1">
          Ville *
          {value.country && !loadingCities && (
            <button
              type="button"
              className="text-xs text-primary underline font-normal ml-2"
              onClick={() => {
                setCityFreeText(!cityFreeText);
                onChange({ ...value, city: "" });
              }}
            >
              {cityFreeText ? "← Liste" : "Saisir manuellement"}
            </button>
          )}
        </span>
        {cityFreeText ? (
          <>
            <input
              required
              type="text"
              placeholder="Nom de la ville…"
              disabled={disabled}
              className="form-input"
              value={value.city}
              onChange={(e) => onChange({ ...value, city: e.target.value })}
            />
            {cityApiEmpty && (
              <p className="text-xs text-muted-foreground mt-1">
                Aucune ville trouvée via l'API pour ce pays.
              </p>
            )}
          </>
        ) : (
          <select
            required
            disabled={disabled || !value.country || loadingCities}
            className="form-select"
            value={value.city}
            onChange={(e) => {
              if (e.target.value === "__other__") {
                setCityFreeText(true);
                onChange({ ...value, city: "" });
              } else {
                onChange({ ...value, city: e.target.value });
              }
            }}
          >
            <option value="">{loadingCities ? "Chargement…" : "— Choisir —"}</option>
            {cities.map((city) => (
              <option key={city} value={city}>{city}</option>
            ))}
            {!loadingCities && cities.length > 0 && (
              <option value="__other__">✏ Saisir manuellement…</option>
            )}
          </select>
        )}
      </div>

      {/* FUSEAU HORAIRE */}
      <div className="block text-sm">
        <span className="font-medium text-muted-foreground flex items-center justify-between mb-1">
          Fuseau horaire *
          {value.country && !loadingTz && (
            <button
              type="button"
              className="text-xs text-primary underline font-normal ml-2"
              onClick={() => {
                setTzFreeText(!tzFreeText);
                onChange({ ...value, timezone: "" });
              }}
            >
              {tzFreeText ? "← Liste" : "Saisir manuellement"}
            </button>
          )}
        </span>
        {tzFreeText ? (
          <>
            <input
              required
              type="text"
              placeholder="ex: Africa/Brazzaville"
              disabled={disabled}
              className="form-input"
              value={value.timezone}
              onChange={(e) => onChange({ ...value, timezone: e.target.value })}
            />
            {tzApiEmpty && (
              <p className="text-xs text-muted-foreground mt-1">
                Aucun fuseau trouvé via l'API pour ce pays.
              </p>
            )}
          </>
        ) : (
          <select
            required
            disabled={disabled || !value.country || loadingTz}
            className="form-select"
            value={value.timezone}
            onChange={(e) => {
              if (e.target.value === "__other__") {
                setTzFreeText(true);
                onChange({ ...value, timezone: "" });
              } else {
                onChange({ ...value, timezone: e.target.value });
              }
            }}
          >
            <option value="">{loadingTz ? "Chargement…" : "— Choisir —"}</option>
            {timezones.map((tz) => (
              <option key={tz} value={tz}>{tz}</option>
            ))}
            {!loadingTz && timezones.length > 0 && (
              <option value="__other__">✏ Saisir manuellement…</option>
            )}
          </select>
        )}
      </div>
    </div>
  );
}
