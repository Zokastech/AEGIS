// AEGIS — zokastech.fr — Apache 2.0 / MIT

package policy

import (
	"fmt"
	"io/fs"
	"os"
	"path"
	"path/filepath"
	"strings"

	"gopkg.in/yaml.v3"
)

// LoadFile loads a single YAML policy file.
func LoadFile(path string) (*PolicyDocument, error) {
	b, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	var p PolicyDocument
	if err := yaml.Unmarshal(b, &p); err != nil {
		return nil, fmt.Errorf("policy %s: %w", path, err)
	}
	p.Normalize()
	if p.Name == "" {
		return nil, fmt.Errorf("policy %s: name requis", path)
	}
	return &p, nil
}

// LoadDir loads all *.yaml in a directory (non-recursive).
func LoadDir(dir string) (map[string]*PolicyDocument, error) {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil, err
	}
	out := make(map[string]*PolicyDocument)
	for _, e := range entries {
		if e.IsDir() || !strings.HasSuffix(strings.ToLower(e.Name()), ".yaml") {
			continue
		}
		p, err := LoadFile(filepath.Join(dir, e.Name()))
		if err != nil {
			return nil, err
		}
		out[p.Name] = p
	}
	return out, nil
}

// LoadFS loads policies from an fs.FS (e.g. embed).
func LoadFS(fsys fs.FS, dir string) (map[string]*PolicyDocument, error) {
	entries, err := fs.ReadDir(fsys, dir)
	if err != nil {
		return nil, err
	}
	out := make(map[string]*PolicyDocument)
	for _, e := range entries {
		if e.IsDir() || !strings.HasSuffix(strings.ToLower(e.Name()), ".yaml") {
			continue
		}
		fpath := path.Join(dir, e.Name())
		b, err := fs.ReadFile(fsys, fpath)
		if err != nil {
			return nil, err
		}
		var doc PolicyDocument
		if err := yaml.Unmarshal(b, &doc); err != nil {
			return nil, fmt.Errorf("policy %s: %w", fpath, err)
		}
		doc.Normalize()
		if doc.Name == "" {
			return nil, fmt.Errorf("policy %s: name requis", fpath)
		}
		out[doc.Name] = &doc
	}
	return out, nil
}
