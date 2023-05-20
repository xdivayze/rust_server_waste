import clip
import matplotlib.pyplot as plt
import torch
from PIL import Image


class ClipController:
    def __init__(self):
        self.device = 'cuda' if torch.cuda.is_available() else 'cpu'
        self.model, self.preprocess = clip.load("ViT-B/32", device=self.device)
        self.prompts = ["plastic", "carton", "textile", "food", "paper", "metal", "glass", "packing", "uma thurman",
                        "medical",
                        "battery", "hazardous", "organic", "electronic", "wood", "mixed", "other"]

        self.text = clip.tokenize(self.prompts).to(self.device)
        print('ClipController initialized')

    def get_clip_features(self, image):
        image = self.preprocess(Image.open(image)).unsqueeze(0).to(self.device)
        print(image.shape)
        plt.figure(figsize=(20, 20))
        plt.imshow(image[0].permute(1, 2, 0).cpu().numpy())
        plt.show()

        with torch.no_grad():
            logits_per_image, logits_per_text = self.model(image, self.text)
            probs = logits_per_image.softmax(dim=-1).cpu().numpy()

        # print all label's probabilities
        # for i, prompt in enumerate(self.prompts):
        #     print(f'{prompt} : {probs[0][i]}')

        return self.prompts[probs.argmax()], probs.max()
