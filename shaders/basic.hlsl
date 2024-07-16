struct VsInput
{
    [[vk::location(0)]] float4 position : POSITION0;
    [[vk::location(1)]] float4 color : COLOR0;
};

struct VsOutput
{
    float4 position : SV_Position;
    [[vk::location(0)]] float4 color : COLOR0;
};

VsOutput VSMain(VsInput input) {
    VsOutput output = (VsOutput)0;

    output.position = input.position;
    output.color = input.color;
    return output;
}

float4 PSMain(VsOutput input) : COLOR
{
    return input.color;
}
