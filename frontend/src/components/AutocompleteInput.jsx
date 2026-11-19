import { useState, useEffect, useRef, useCallback } from 'react';
import { TextInput } from '@astryxdesign/core/TextInput';
import { Stack } from '@astryxdesign/core/Stack';
import { Text } from '@astryxdesign/core/Text';
import { Card } from '@astryxdesign/core/Card';

export function AutocompleteInput({ value, onChange, suggestions, onSelect, placeholder }) {
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [highlightIndex, setHighlightIndex] = useState(-1);
  const inputRef = useRef(null);

  const filtered = suggestions.filter(s => 
    value.toLowerCase().includes(s.toLowerCase()) || value === ''
  ).slice(0, 8);

  useEffect(() => {
    if (value.includes('@') || value.includes('#')) {
      setShowSuggestions(true);
    } else {
      setShowSuggestions(false);
      setHighlightIndex(-1);
    }
  }, [value]);

  const handleKeyDown = (e) => {
    if (!showSuggestions || filtered.length === 0) return;

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setHighlightIndex(prev => Math.min(prev + 1, filtered.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setHighlightIndex(prev => Math.max(prev - 1, -1));
    } else if (e.key === 'Tab' || e.key === 'Enter') {
      e.preventDefault();
      if (highlightIndex >= 0) {
        onSelect(filtered[highlightIndex]);
        setShowSuggestions(false);
        setHighlightIndex(-1);
      }
    } else if (e.key === 'Escape') {
      setShowSuggestions(false);
      setHighlightIndex(-1);
    }
  };

  const handleSelect = (suggestion) => {
    onSelect(suggestion);
    setShowSuggestions(false);
    setHighlightIndex(-1);
  };

  return (
    <Stack padding={0} width="100%">
      <TextInput
        ref={inputRef}
        label={placeholder || ''}
        value={value}
        onChange={onChange}
        onKeyDown={handleKeyDown}
      />
      {showSuggestions && filtered.length > 0 && (
        <Card padding={1} elevation="med" style={{ position: 'absolute', zIndex: 1000, width: '100%', maxHeight: 200, overflow: 'auto', marginTop: '0.25rem' }}>
          <Stack gap={0}>
            {filtered.map((suggestion, idx) => (
              <Button
                key={suggestion}
                label={suggestion}
                variant="ghost"
                size="sm"
                style={{
                  width: '100%',
                  textAlign: 'left',
                  backgroundColor: highlightIndex === idx ? '#e3f2fd' : 'transparent',
                  borderRadius: 6,
                  cursor: 'pointer'
                }}
                onClick={() => handleSelect(suggestion)}
              />
            ))}
          </Stack>
        </Card>
      )}
    </Stack>
  );
}

import { Button } from '@astryxdesign/core/Button';