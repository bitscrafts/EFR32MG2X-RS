// Minimal blink for baseline testing
void setup() {
  pinMode(LED_BUILTIN, OUTPUT);
}

void loop() {
  digitalWrite(LED_BUILTIN, LOW);   // LED ON (active low)
  delay(1000);
  digitalWrite(LED_BUILTIN, HIGH);  // LED OFF
  delay(1000);
}
