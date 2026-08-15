import diffTemplate from "../../src/dmx/templates/diff.mustache?raw";
import enumTemplate from "../../src/dmx/templates/enum.mustache?raw";
import lerpTemplate from "../../src/dmx/templates/lerp.mustache?raw";
import modelTemplate from "../../src/dmx/templates/model.mustache?raw";
import unionTemplate from "../../src/dmx/templates/union.mustache?raw";

export type MacroId = "model" | "enum" | "union" | "diff" | "lerp";

export interface PlaygroundSample {
  readonly id: MacroId;
  readonly name: string;
  readonly description: string;
  readonly source: string;
  readonly template: string;
}

const MODEL_SAMPLE: PlaygroundSample = {
  id: "model",
  name: "Profile",
  description: "Total JSON, equality, hashCode, toString and typed copyWith.",
  template: modelTemplate,
  source: `import 'package:dmx/dmx.dart';

@dmx('model', {'fieldRename': 'snake'})
class Profile {
  const Profile({
    required this.id,
    required this.displayName,
    this.email,
    this.tags = const [],
  });

  final String id;
  final String displayName;
  final String? email;
  final List<String> tags;
}
`,
};

export const PLAYGROUND_SAMPLES: readonly PlaygroundSample[] = [
  MODEL_SAMPLE,
  {
    id: "enum",
    name: "DeliveryState",
    description: "Wire names, human labels and an explicit unknown fallback.",
    template: enumTemplate,
    source: `import 'package:dmx/dmx.dart';

@dmx('enum', {'fieldRename': 'snake', 'unknown': DeliveryState.unknown})
enum DeliveryState {
  preparing,
  outForDelivery,
  delivered,
  unknown;
}
`,
  },
  {
    id: "union",
    name: "PaymentState",
    description: "A tagged sealed family with exhaustive decoding and matching.",
    template: unionTemplate,
    source: `import 'package:dmx/dmx.dart';

@dmx('union', {'discriminator': 'type', 'fieldRename': 'snake'})
sealed class PaymentState {
  const PaymentState();
}

final class Pending extends PaymentState {
  const Pending();

  static Result<Pending, DecodeError> fromJson(
    Object? json, [
    String path = 'Pending',
  ]) => Ok(const Pending());

  @override
  Map<String, dynamic> toJson() => {'type': 'pending'};
}

final class PaymentCaptured extends PaymentState {
  const PaymentCaptured();

  static Result<PaymentCaptured, DecodeError> fromJson(
    Object? json, [
    String path = 'PaymentCaptured',
  ]) => Ok(const PaymentCaptured());

  @override
  Map<String, dynamic> toJson() => {'type': 'payment_captured'};
}
`,
  },
  {
    id: "diff",
    name: "InventoryItem",
    description: "Structural field changes represented as immutable data.",
    template: diffTemplate,
    source: `import 'package:dmx/dmx.dart';

@dmx('diff')
class InventoryItem {
  const InventoryItem({
    required this.sku,
    required this.onHand,
    this.labels = const [],
  });

  final String sku;
  final int onHand;
  final List<String> labels;
}
`,
  },
  {
    id: "lerp",
    name: "MotionTokens",
    description: "Composable interpolation for motion and design-token values.",
    template: lerpTemplate,
    source: `import 'package:dmx/dmx.dart';

@dmx('lerp')
class MotionTokens {
  const MotionTokens({
    required this.opacity,
    required this.offset,
    required this.duration,
    required this.curve,
  });

  final double opacity;
  final int offset;
  final Duration duration;
  final String curve;
}
`,
  },
];

export function sampleById(id: string): PlaygroundSample {
  return PLAYGROUND_SAMPLES.find((sample) => sample.id === id) ?? MODEL_SAMPLE;
}
