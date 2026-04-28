import { ProductDetailView } from "@/components/features/product-detail-view";
import { getProduct } from "@/lib/repository";

export default async function ProductDetailPage({
  params,
}: {
  params: Promise<{ productId: string }>;
}) {
  const { productId } = await params;
  const detail = await getProduct(productId);

  return (
    <ProductDetailView
      product={detail.product}
      fields={detail.fields}
      rows={detail.rows}
    />
  );
}
