import { isRouteErrorResponse, useRouteError } from "react-router-dom";

export default function RouteErrorBoundary() {
  const error = useRouteError();
  const message = isRouteErrorResponse(error)
    ? error.statusText
    : "We could not load this page.";

  return (
    <main role="alert" style={{ padding: "2rem" }}>
      <h1>Something went wrong</h1>
      <p>{message}</p>
    </main>
  );
}
