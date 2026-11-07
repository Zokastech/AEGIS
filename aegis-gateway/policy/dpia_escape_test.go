// AEGIS — zokastech.fr — Apache 2.0 / MIT

package policy

import "testing"

func TestHTMLEscapeAllSpecials(t *testing.T) {
	got := htmlEscape(`<img src=x onerror='alert(1)'>&"`)
	want := "&lt;img src=x onerror=&#39;alert(1)&#39;&gt;&amp;&#34;"
	if got != want {
		t.Fatalf("got %q want %q", got, want)
	}
}
