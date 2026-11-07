// AEGIS — zokastech.fr — Apache 2.0 / MIT

// Command aegis-keygen creates, rotates and revokes gateway API keys.
//
// The plaintext secret is printed ONCE on stdout; only SHA-256(pepper:secret) is stored.
// The pepper is read from AEGIS_API_KEY_PEPPER (recommended) or from the file itself.
//
//	AEGIS_API_KEY_PEPPER=$(openssl rand -hex 32) \
//	  aegis-keygen -file /var/lib/aegis/api_keys.yaml -role admin -id ops-admin-1
//	aegis-keygen -file … -revoke ops-admin-1
//	aegis-keygen -file … -rotate ops-admin-1 -id ops-admin-2
package main

import (
	"crypto/rand"
	"encoding/base64"
	"flag"
	"fmt"
	"os"
	"strings"

	"github.com/zokastech/aegis/aegis-gateway/auth/apikey"
	"github.com/zokastech/aegis/aegis-gateway/rbac"
)

func main() {
	if err := run(os.Args[1:]); err != nil {
		fmt.Fprintln(os.Stderr, "aegis-keygen:", err)
		os.Exit(1)
	}
}

func run(args []string) error {
	fs := flag.NewFlagSet("aegis-keygen", flag.ContinueOnError)
	file := fs.String("file", "config/api_keys.yaml", "fichier YAML des clés")
	id := fs.String("id", "", "identifiant de la nouvelle clé (défaut : aléatoire)")
	role := fs.String("role", rbac.RoleViewer, "rôle : viewer | operator | admin | auditor")
	revoke := fs.String("revoke", "", "révoquer la clé portant cet identifiant")
	rotate := fs.String("rotate", "", "faire tourner la clé portant cet identifiant (nouvelle clé = -id)")
	list := fs.Bool("list", false, "lister les identifiants (jamais les secrets)")
	if err := fs.Parse(args); err != nil {
		return err
	}
	store, err := apikey.LoadFileStore(*file, os.Getenv("AEGIS_API_KEY_PEPPER"))
	if err != nil {
		return err
	}
	switch {
	case *list:
		for _, k := range store.Keys {
			state := "active"
			if k.Revoked {
				state = "revoked"
			}
			fmt.Printf("%s\t%s\t%s\t%s\n", k.ID, k.Role, k.CreatedAtRFC3339, state)
		}
		return nil
	case *revoke != "":
		if err := store.Revoke(*revoke); err != nil {
			return err
		}
		fmt.Fprintf(os.Stderr, "clé %q révoquée\n", *revoke)
		return nil
	}
	if !validRole(*role) {
		return fmt.Errorf("rôle %q inconnu", *role)
	}
	newID := strings.TrimSpace(*id)
	if newID == "" {
		newID = "key-" + randomToken(6)
	}
	secret := "aegis_" + randomToken(32)
	if *rotate != "" {
		if err := store.Rotate(*rotate, newID, secret); err != nil {
			return err
		}
	} else if err := store.Generate(newID, *role, secret); err != nil {
		return err
	}
	fmt.Fprintf(os.Stderr, "clé %q créée (stockée hashée dans %s) — secret affiché une seule fois :\n", newID, *file)
	fmt.Println(secret)
	return nil
}

func validRole(r string) bool {
	switch r {
	case rbac.RoleViewer, rbac.RoleOperator, rbac.RoleAdmin, rbac.RoleAuditor:
		return true
	}
	return false
}

func randomToken(n int) string {
	b := make([]byte, n)
	if _, err := rand.Read(b); err != nil {
		panic(err)
	}
	return base64.RawURLEncoding.EncodeToString(b)
}
