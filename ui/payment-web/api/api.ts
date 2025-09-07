// Add to transactionApi
getHistory: async (
  limit = 20,
  offset = 0,
  search?: string,
  type?: 'sent' | 'received'
): Promise<ApiResponse<TransactionItem[]>> => {
  try {
    const token = localStorage.getItem('access_token');
    const params: Record<string, any> = { limit, offset };
    if (search) params.search = search;
    if (type) params.type = type;

    const res = await apiClient.get('/transactions', {
      headers: { Authorization: `Bearer ${token}` },
      params,
    });
    return {  res.data };
  } catch (error) {
    return { data: null, error: 'Failed to fetch history' };
  }
},