import React from "react";

export function SearchStorefrontPage(): React.JSX.Element {
  return (
    <section style={{ border: "1px solid #d4d4d8", borderRadius: 28, padding: 28 }}>
      <div style={{ fontSize: 12, textTransform: "uppercase", letterSpacing: "0.14em", color: "#71717a" }}>
        search
      </div>
      <h2 style={{ marginTop: 10, fontSize: 32 }}>Search experiences start here</h2>
      <p style={{ marginTop: 12, color: "#52525b", maxWidth: 760 }}>
        Next storefront scaffold for rustok-search. It should evolve in parallel with the Leptos
        storefront package on the same query, filter, suggestion, and result contracts.
      </p>
    </section>
  );
}

export default SearchStorefrontPage;
